import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

abstract interface class NostrSocialPort {
  Future<Set<ProfileId>> loadBlockedProfiles();

  Future<Set<ProfileId>> loadFollowedProfiles();

  Future<bool> toggleBlock(ProfileId profileId);

  Future<bool> toggleFollow(ProfileId profileId);
}
