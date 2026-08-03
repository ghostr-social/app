import 'dart:developer';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/settings/domain/relay_url.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_query_port.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_mapper.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_outbox_directory.dart';
import 'package:ghostr/platform/nostr/ndk_video_event_selection.dart';
import 'package:ghostr/platform/nostr/video_discovery_queries.dart';
import 'package:ndk/ndk.dart';

class NdkNostrVideoEventQuery implements NostrVideoEventQueryPort {
  NdkNostrVideoEventQuery(
    this._ndk, {
    List<RelayUrl> searchRelays = const [],
    NdkNostrOutboxDirectory? outbox,
  })  : _searchRelayUrls = List<String>.unmodifiable(
          searchRelays.map((relay) => relay.value),
        ),
        _outbox = outbox;

  static const _feedTimeout = Duration(seconds: 5);
  // Search relays keep answering after the fast ones went quiet; the extra
  // seconds are where the long tail of matches comes from.
  static const _discoveryTimeout = Duration(seconds: 8);

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
      final queries = videoDiscoveryQueries(
        authorPublicKeys: authorPublicKeys,
        searchQuery: searchQuery,
        hashtags: hashtags,
        olderThan: olderThan,
      );
      final relays = await _relayTargets(queries.first);
      final results = await Future.wait([
        _queryResponse(queries.first, relays).future,
        ...queries.skip(1).map((query) => _additiveEvents(query, relays)),
      ]);
      return acceptNostrVideoEvents(
        events: results.expand((events) => events),
        queries: queries,
      );
    } on Object catch (error, stackTrace) {
      throw _failure('Could not load Nostr videos.', error, stackTrace);
    }
  }

  // Note results only ever widen the pool; their hiccups must not sink the
  // primary video results.
  Future<List<Nip01Event>> _additiveEvents(
    NostrEventQuery query,
    List<String>? relays,
  ) async {
    try {
      return await _queryResponse(query, relays).future;
    } on Object catch (error, stackTrace) {
      log(
        'Skipping a failed additive discovery query.',
        name: 'ghostr.nostr.video-query',
        error: error,
        stackTrace: stackTrace,
      );
      return const <Nip01Event>[];
    }
  }

  // NIP-50 terms only work on relays that index for search. Hashtag queries
  // hit those same deep indexes merged with the outbox; everything else
  // routes to the outbox relays where the wanted authors actually publish.
  Future<List<String>?> _relayTargets(NostrEventQuery query) async {
    if (query.search != null) {
      return _searchRelayUrls.isEmpty ? null : _searchRelayUrls;
    }
    final outbox = await _outboxTargets(query);
    if (query.tagFilters.isEmpty) return outbox;
    final merged = {..._searchRelayUrls, ...?outbox};
    return merged.isEmpty ? null : merged.toList();
  }

  Future<List<String>?> _outboxTargets(NostrEventQuery query) async {
    final outbox = _outbox;
    if (outbox == null) return null;
    final relays = query.authors.isEmpty
        ? await outbox.discoveryRelayUrls()
        : await outbox.authorWriteRelayUrls(query.authors.toSet());
    return relays.isEmpty ? null : relays;
  }

  NdkResponse _queryResponse(NostrEventQuery query, List<String>? relays) {
    final name = _requestName(query);
    final timeout = _isDiscovery(query) ? _discoveryTimeout : _feedTimeout;
    final filter = _mapper.toFilter(query);
    if (relays == null) {
      return _ndk.requests.query(name: name, timeout: timeout, filter: filter);
    }
    return _ndk.requests.query(
      name: name,
      timeout: timeout,
      filter: filter,
      explicitRelays: relays,
    );
  }

  bool _isDiscovery(NostrEventQuery query) {
    return query.search != null || query.tagFilters.isNotEmpty;
  }

  String _requestName(NostrEventQuery query) {
    final notesOnly = query.kinds.length == 1 && query.kinds.single.value == 1;
    if (_isDiscovery(query)) {
      return notesOnly ? 'ghostr-note-search' : 'ghostr-video-search';
    }
    return notesOnly ? 'ghostr-note-feed' : 'ghostr-video-feed';
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
