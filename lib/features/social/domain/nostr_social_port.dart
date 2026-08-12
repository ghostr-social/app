import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/social/domain/follow_outcome.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

abstract interface class NostrSocialPort {
  NostrSocialPort snapshotForActiveAccount();

  NostrPublicKeyHex get accountPublicKey;

  Future<Set<ProfileId>> loadBlockedProfiles();

  Future<Set<ProfileId>> loadFollowedProfiles();

  Future<FollowOutcome> follow(ProfileId profileId);

  Future<bool> toggleBlock(ProfileId profileId);

  Future<bool> toggleFollow(ProfileId profileId);
}
