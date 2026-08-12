import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_request.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_state.dart';

ProfileState initialProfileState(ProfileRequest request) {
  if (request.viewer.id != request.profileId) {
    return const ProfileState.loading();
  }
  return ProfileState.ready(
    ProfileDetails(
      profile: request.viewer,
      posts: const [],
      statistics: ProfileStatistics(totalLikes: 0, followingCount: 0),
      relationship: ProfileRelationship(
        isFollowing: false,
        isBlocked: false,
        isCurrentUser: true,
      ),
    ),
    isRefreshing: true,
  );
}
