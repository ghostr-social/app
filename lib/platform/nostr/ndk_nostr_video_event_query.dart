import 'dart:developer';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/nostr/nostr_query_result_policy.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_query_port.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_event_mapper.dart';
import 'package:ndk/ndk.dart';

typedef _MappedVideoEvent = ({
  Nip01Event transport,
  NostrEventRecord record,
});

class NdkNostrVideoEventQuery implements NostrVideoEventQueryPort {
  NdkNostrVideoEventQuery(this._ndk);

  static const _videoKinds = [21, 22, 34235, 34236];

  final Ndk _ndk;
  final NdkNostrEventMapper _mapper = const NdkNostrEventMapper();

  @override
  Future<List<Nip01Event>> loadVideoEvents({
    Set<NostrPublicKeyHex>? authorPublicKeys,
    String? searchQuery,
  }) async {
    try {
      final query = _videoQuery(authorPublicKeys);
      final response = _ndk.requests.query(
        name: 'ghostr-video-feed',
        timeout: const Duration(seconds: 5),
        filter: _videoFilter(query, searchQuery),
      );
      final events = await response.future;
      return _acceptedVideoEvents(events, query);
    } on Object catch (error, stackTrace) {
      throw _failure('Could not load Nostr videos.', error, stackTrace);
    }
  }

  NostrEventQuery _videoQuery(Set<NostrPublicKeyHex>? authorPublicKeys) {
    return NostrEventQuery(
      kinds: _videoKinds,
      scope: NostrEventQueryScope(
        authors: authorPublicKeys?.toList() ?? const <NostrPublicKeyHex>[],
      ),
      limit: 80,
    );
  }

  Filter _videoFilter(NostrEventQuery query, String? searchQuery) {
    // NIP-50 search matching is relay-defined; only structural fields can be
    // revalidated locally without inventing incompatible search semantics.
    return _mapper.toFilter(query)..search = searchQuery;
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
