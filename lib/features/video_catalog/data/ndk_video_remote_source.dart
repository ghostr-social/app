import 'dart:developer';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_mapper.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_event_query_port.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ndk/ndk.dart';

class NdkVideoRemoteSource implements RemoteVideoSource {
  NdkVideoRemoteSource(
    this._queryPort, {
    NostrVideoEventMapper mapper = const NostrVideoEventMapper(),
  }) : _mapper = mapper;

  final NostrVideoEventQueryPort _queryPort;
  final NostrVideoEventMapper _mapper;

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) async {
    final authors = _decodeCreatorIds(creatorIds);
    if (creatorIds != null && authors!.isEmpty) return const [];
    final events = _canonicalEvents(await _queryPort.loadVideoEvents(
      authorPublicKeys: authors,
      searchQuery: searchQuery,
      hashtags: hashtags,
    ));
    final metadata = await _loadMetadata(events);
    final posts = <VideoPost>[];
    for (final event in events) {
      try {
        posts.add(_mapper.map(event, metadata[event.pubKey]));
      } on AppFailure catch (failure, stackTrace) {
        log(
          'Skipping a malformed Nostr video event.',
          name: 'ghostr.video.nostr',
          error: failure,
          stackTrace: stackTrace,
        );
        continue;
      }
    }
    return posts;
  }

  Set<NostrPublicKeyHex>? _decodeCreatorIds(Set<ProfileId>? creatorIds) {
    if (creatorIds == null) return null;
    final publicKeys = <NostrPublicKeyHex>{};
    for (final creatorId in creatorIds) {
      try {
        publicKeys.add(NostrPublicKeyHex.parse(Nip19.decode(creatorId.value)));
      } on Object catch (error, stackTrace) {
        log(
          'Skipping a non-Nostr creator identifier.',
          name: 'ghostr.video.nostr',
          error: error,
          stackTrace: stackTrace,
        );
        continue;
      }
    }
    return publicKeys;
  }

  Future<Map<String, Metadata>> _loadMetadata(List<Nip01Event> events) async {
    final publicKeys = _metadataPublicKeys(events);
    if (publicKeys.isEmpty) return const {};
    try {
      final batch = await _queryPort.loadMetadataBatch(publicKeys);
      return {for (final entry in batch.entries) entry.key.value: entry.value};
    } on Object catch (error, stackTrace) {
      log(
        'Creator metadata batch could not be loaded.',
        name: 'ghostr.video.nostr',
        error: error,
        stackTrace: stackTrace,
      );
      return const {};
    }
  }

  Set<NostrPublicKeyHex> _metadataPublicKeys(List<Nip01Event> events) {
    final publicKeys = <NostrPublicKeyHex>{};
    for (final event in events) {
      try {
        publicKeys.add(NostrPublicKeyHex.parse(event.pubKey));
      } on FormatException catch (error, stackTrace) {
        log(
          'Skipping malformed creator metadata identity.',
          name: 'ghostr.video.nostr',
          error: error,
          stackTrace: stackTrace,
        );
      }
    }
    return publicKeys;
  }

  List<Nip01Event> _canonicalEvents(List<Nip01Event> events) {
    final selected = <String, Nip01Event>{};
    for (final event in events) {
      final key = _eventCoordinate(event);
      final current = selected[key];
      if (current == null || _isNewer(event, current)) selected[key] = event;
    }
    final canonical = selected.values.toList();
    canonical.sort(_compareNewest);
    return canonical;
  }

  String _eventCoordinate(Nip01Event event) {
    if (event.kind < 30000 || event.kind >= 40000) return event.id;
    final identifier = event.tags
        .where((tag) => tag.firstOrNull == 'd')
        .firstOrNull
        ?.elementAtOrNull(1);
    return identifier == null || identifier.trim().isEmpty
        ? event.id
        : '${event.kind}:${event.pubKey}:$identifier';
  }

  bool _isNewer(Nip01Event incoming, Nip01Event current) {
    return incoming.createdAt > current.createdAt ||
        (incoming.createdAt == current.createdAt &&
            incoming.id.compareTo(current.id) < 0);
  }

  int _compareNewest(Nip01Event left, Nip01Event right) {
    final time = right.createdAt.compareTo(left.createdAt);
    return time == 0 ? left.id.compareTo(right.id) : time;
  }
}
