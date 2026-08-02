import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

abstract interface class SocialGraphRepository {
  Future<Set<ProfileId>> loadFollowedProfiles();

  Future<Set<ProfileId>> loadBlockedProfiles();

  Future<bool> toggleFollow(ProfileId profileId);

  Future<bool> toggleBlock(ProfileId profileId);
}
