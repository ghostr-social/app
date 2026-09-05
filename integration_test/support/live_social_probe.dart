import 'package:ghostr/app/production_nostr_services.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/social/domain/follow_outcome.dart';
import 'package:ghostr/features/social/domain/nostr_social_port.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import 'live_video_log.dart';

ProductionNostrServices liveNostrServices(
  ProductionNostrServices production,
  LiveVideoLog log,
) => ProductionNostrServices(
  ProductionNostrAdapters(
    production.adapters.session,
    LiveSocialProbe(production.adapters.social, log),
  ),
  production.eventClient,
  production.publisher,
  production.profileImageUploader,
);

final class LiveSocialProbe implements NostrSocialPort {
  LiveSocialProbe(this.delegate, this.log);
  final NostrSocialPort delegate;
  final LiveVideoLog log;

  @override
  NostrPublicKeyHex get accountPublicKey => delegate.accountPublicKey;

  @override
  NostrSocialPort snapshotForActiveAccount() =>
      LiveSocialProbe(delegate.snapshotForActiveAccount(), log);

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() =>
      _read('blocked', delegate.loadBlockedProfiles);

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() =>
      _read('following', delegate.loadFollowedProfiles);

  Future<Set<ProfileId>> _read(
    String kind,
    Future<Set<ProfileId>> Function() read,
  ) async {
    final clock = Stopwatch()..start();
    log.add('social_read_started', {'kind': kind});
    try {
      final result = await read();
      log.add('social_read_finished', {
        'kind': kind,
        'count': result.length,
        'durationMs': clock.elapsedMilliseconds,
      });
      return result;
    } on Object catch (error) {
      log.add('social_read_failed', {
        'kind': kind,
        'durationMs': clock.elapsedMilliseconds,
        'error': '$error',
      });
      rethrow;
    }
  }

  @override
  Future<FollowOutcome> follow(ProfileId id) => delegate.follow(id);
  @override
  Future<bool> toggleFollow(ProfileId id) => delegate.toggleFollow(id);
  @override
  Future<bool> toggleBlock(
    ProfileId id, {
    Set<ProfileId> knownBlocked = const {},
  }) => delegate.toggleBlock(id, knownBlocked: knownBlocked);
}
