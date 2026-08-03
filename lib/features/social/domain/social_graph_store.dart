import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

abstract interface class SocialGraphStore {
  SocialGraphStore snapshotForActiveAccount();

  NostrPublicKeyHex get accountPublicKey;

  Future<Set<ProfileId>> loadFollowedProfiles();

  Future<Set<ProfileId>> loadBlockedProfiles();

  Future<void> saveFollowedProfiles(Set<ProfileId> profileIds);

  Future<void> saveBlockedProfiles(Set<ProfileId> profileIds);
}
