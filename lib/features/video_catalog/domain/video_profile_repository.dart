import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';

abstract interface class VideoProfileRepository {
  Future<ProfileDetails> loadProfile(
    ProfileSummary viewer,
    ProfileId profileId,
  );

  Future<bool> toggleFollow(ProfileId profileId);

  Future<bool> toggleBlock(ProfileId profileId);
}
