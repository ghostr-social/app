import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/profile/data/local_profile_metadata_cache.dart';
import 'package:ghostr/features/profile/data/nostr_profile_metadata_mapper.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/profile/domain/profile_metadata_repository.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

final class NostrProfileMetadataRepository
    implements ProfileMetadataRepository {
  NostrProfileMetadataRepository({
    required NostrEventClient client,
    required LocalProfileMetadataCache cache,
    NostrProfileMetadataMapper mapper = const NostrProfileMetadataMapper(),
    int Function() clock = _unixNow,
  }) : _client = client,
       _cache = cache,
       _mapper = mapper,
       _clock = clock;

  final NostrEventClient _client;
  final LocalProfileMetadataCache _cache;
  final NostrProfileMetadataMapper _mapper;
  final int Function() _clock;

  @override
  Future<ProfileSummary?> loadCached(ProfileId profileId) {
    return _cache.read(profileId);
  }

  @override
  Future<ProfileSummary?> refresh(ProfileId profileId) async {
    final event = await _latest(_publicKey(profileId));
    if (event == null) return (await _cache.readSnapshot(profileId))?.profile;
    try {
      final cached = await _cache.readSnapshot(profileId);
      if (_isStale(event, cached)) return cached?.profile;
    } on AppFailure {
      return _cacheEvent(event, profileId);
    }
    return _cacheEvent(event, profileId);
  }

  Future<ProfileSummary> _cacheEvent(
    NostrEventRecord event,
    ProfileId profileId,
  ) async {
    final summary = _mapper.summaryFromEvent(event, profileId);
    await _cache.write(summary, observedAt: event.createdAt);
    return summary;
  }

  @override
  Future<ProfileSummary> save(
    NostrIdentity identity,
    ProfileMetadata metadata,
  ) async {
    _verifyActive(identity);
    final previous = await _latest(identity.publicKeyHex);
    await _client.publish(
      _mapper.toEvent(metadata, previousContent: previous?.content),
      expectedAuthor: identity.publicKeyHex,
    );
    final summary = metadata.toSummary(ProfileId.parse(identity.npub));
    await _cache.write(summary, observedAt: _clock());
    return summary;
  }

  bool _isStale(NostrEventRecord event, CachedProfileMetadata? cached) {
    return cached != null && cached.observedAt >= event.createdAt;
  }

  Future<NostrEventRecord?> _latest(NostrPublicKeyHex author) async {
    final events = await _client.query(
      NostrEventQuery(
        kinds: const [0],
        scope: NostrEventQueryScope(authors: [author]),
        limit: 20,
      ),
    );
    NostrEventRecord? latest;
    for (final event in events) {
      if (latest == null || _isNewer(event, latest)) latest = event;
    }
    return latest;
  }

  bool _isNewer(NostrEventRecord candidate, NostrEventRecord current) {
    if (candidate.createdAt != current.createdAt) {
      return candidate.createdAt > current.createdAt;
    }
    return candidate.id.compareTo(current.id) < 0;
  }

  NostrPublicKeyHex _publicKey(ProfileId id) {
    return NostrPublicKeyHex.parse(NostrNpub.parse(id.value).publicKeyHex);
  }

  void _verifyActive(NostrIdentity identity) {
    if (_client.publicKeyHex != identity.publicKeyHex) {
      throw const AppFailure('The active account changed. Try again.');
    }
  }
}

int _unixNow() => DateTime.now().millisecondsSinceEpoch ~/ 1000;
