import 'dart:developer';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/nostr/nostr_query_result_policy.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_query_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_hashtags.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_mapper.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_outbox_directory.dart';
import 'package:ndk/ndk.dart';

typedef _MappedVideoEvent = ({
  Nip01Event transport,
  NostrEventRecord record,
});

class NdkNostrVideoEventQuery implements NostrVideoEventQueryPort {
  NdkNostrVideoEventQuery(
    this._ndk, {
    List<RelayUrl> searchRelays = const [],
    NdkNostrOutboxDirectory? outbox,
  })  : _searchRelayUrls = List<String>.unmodifiable(
          searchRelays.map((relay) => relay.value),
        ),
        _outbox = outbox;

  static const _videoKinds = [21, 22, 34235, 34236];
  static const _timeout = Duration(seconds: 5);

  final Ndk _ndk;
  final List<String> _searchRelayUrls;
  final NdkNostrOutboxDirectory? _outbox;
  final NdkNostrEventMapper _mapper = const NdkNostrEventMapper();

  @override
  Future<List<Nip01Event>> loadVideoEvents({
    Set<NostrPublicKeyHex>? authorPublicKeys,
    String? searchQuery,
    Set<String>? hashtags,
    DateTime? olderThan,
  }) async {
    try {
      final query = _videoQuery(
        authorPublicKeys,
        hashtags,
        searchQuery: searchQuery,
        olderThan: olderThan,
      );
      final events =
          await _queryResponse(query, await _relayTargets(query)).future;
      return _acceptedVideoEvents(events, query);
    } on Object catch (error, stackTrace) {
      throw _failure('Could not load Nostr videos.', error, stackTrace);
    }
  }

  // NIP-50 terms only work on relays that index for search; everything else
  // routes to the outbox relays where the wanted authors actually publish.
  Future<List<String>?> _relayTargets(NostrEventQuery query) async {
    if (query.search != null) {
      return _searchRelayUrls.isEmpty ? null : _searchRelayUrls;
    }
    final outbox = _outbox;
    if (outbox == null) return null;
    final relays = query.authors.isEmpty
        ? await outbox.discoveryRelayUrls()
        : await outbox.authorWriteRelayUrls(query.authors.toSet());
    return relays.isEmpty ? null : relays;
  }

  NdkResponse _queryResponse(NostrEventQuery query, List<String>? relays) {
    final name =
        query.search == null ? 'ghostr-video-feed' : 'ghostr-video-search';
    final filter = _mapper.toFilter(query);
    if (relays == null) {
      return _ndk.requests.query(name: name, timeout: _timeout, filter: filter);
    }
    return _ndk.requests.query(
      name: name,
      timeout: _timeout,
      filter: filter,
      explicitRelays: relays,
    );
  }

  NostrEventQuery _videoQuery(
    Set<NostrPublicKeyHex>? authorPublicKeys,
    Set<String>? hashtags, {
    String? searchQuery,
    DateTime? olderThan,
  }) {
    return NostrEventQuery(
      kinds: _videoKinds,
      scope: NostrEventQueryScope(
        authors: authorPublicKeys?.toList() ?? const <NostrPublicKeyHex>[],
      ),
      tagFilters: [
        if (hashtags != null && hashtags.isNotEmpty)
          NostrTagFilter(
            name: 't',
            values:
                hashtags.expand(hashtagQueryVariants).toSet().toList(),
          ),
      ],
      // Hashtag queries widen the candidate pool because feed relays only
      // ever return the newest events.
      limit: searchQuery != null || hashtags != null ? 200 : 80,
      until: olderThan == null
          ? null
          : olderThan.toUtc().millisecondsSinceEpoch ~/ 1000,
      search: searchQuery,
    );
  }

  List<Nip01Event> _acceptedVideoEvents(
    Iterable<Nip01Event> events,
    NostrEventQuery query,
  ) {
    final unique = _newestUnique(events);
    final selected = selectNostrQueryResults(
      events: unique.values.map((event) => event.record),
      queries: <NostrEventQuery>[query],
    );
    return selected.map((record) => unique[record.id]!.transport).toList();
  }

  Map<NostrEventId, _MappedVideoEvent> _newestUnique(
    Iterable<Nip01Event> events,
  ) {
    final ordered = events.indexed.toList()..sort(_newestFirst);
    final unique = <NostrEventId, _MappedVideoEvent>{};
    for (final (_, transport) in ordered) {
      final record = _mapper.toRecord(transport);
      unique.putIfAbsent(
          record.id, () => (transport: transport, record: record));
    }
    return unique;
  }

  int _newestFirst(
    (int, Nip01Event) left,
    (int, Nip01Event) right,
  ) {
    final recency = right.$2.createdAt.compareTo(left.$2.createdAt);
    return recency == 0 ? left.$1.compareTo(right.$1) : recency;
  }

  @override
  Future<Map<NostrPublicKeyHex, Metadata>> loadMetadataBatch(
    Set<NostrPublicKeyHex> publicKeys,
  ) async {
    if (publicKeys.isEmpty) return const {};
    try {
      final metadata = await _ndk.metadata.loadMetadatas(
        publicKeys.map((publicKey) => publicKey.value).toList(),
        null,
      );
      return _validMetadata(metadata, publicKeys);
    } on Object catch (error, stackTrace) {
      throw _failure(
          'Could not load Nostr profile metadata.', error, stackTrace);
    }
  }

  Map<NostrPublicKeyHex, Metadata> _validMetadata(
    List<Metadata> metadata,
    Set<NostrPublicKeyHex> requested,
  ) {
    final valid = <NostrPublicKeyHex, Metadata>{};
    for (final entry in metadata) {
      try {
        final publicKey = NostrPublicKeyHex.parse(entry.pubKey);
        if (requested.contains(publicKey)) valid[publicKey] = entry;
      } on FormatException catch (error, stackTrace) {
        log(
          'Skipping malformed Nostr profile metadata.',
          name: 'ghostr.nostr.video-query',
          error: error,
          stackTrace: stackTrace,
        );
      }
    }
    return valid;
  }

  AppFailure _failure(String message, Object error, StackTrace stackTrace) {
    return translatedBoundaryFailure(
      source: 'ghostr.nostr.video-query',
      message: message,
      error: error,
      stackTrace: stackTrace,
    );
  }
}
