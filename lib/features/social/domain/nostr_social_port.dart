import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/social/domain/follow_outcome.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

abstract interface class NostrSocialPort {
  NostrSocialPort snapshotForActiveAccount();

  NostrPublicKeyHex get accountPublicKey;

  Future<Set<ProfileId>> loadBlockedProfiles();

  Future<Set<ProfileId>> loadFollowedProfiles();

  Future<FollowOutcome> follow(ProfileId profileId);

  /// Toggles [profileId] on the mute list. [knownBlocked] carries every
  /// block this device already knows about, so a publish never shrinks the
  /// list just because the relays failed to return it.
  Future<bool> toggleBlock(
    ProfileId profileId, {
    Set<ProfileId> knownBlocked,
  });

  Future<bool> toggleFollow(ProfileId profileId);
}
