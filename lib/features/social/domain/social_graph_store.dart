import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

abstract interface class SocialGraphStore {
  Future<Set<ProfileId>> loadFollowedProfiles();

  Future<Set<ProfileId>> loadBlockedProfiles();

  Future<void> saveFollowedProfiles(Set<ProfileId> profileIds);

  Future<void> saveBlockedProfiles(Set<ProfileId> profileIds);
}
