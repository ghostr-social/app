import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class ProfileSocialContext {
  ProfileSocialContext({
    required this.viewer,
    required this.targetId,
    required Set<ProfileId> followed,
    required Set<ProfileId> blocked,
  })  : followed = Set<ProfileId>.unmodifiable(followed),
        blocked = Set<ProfileId>.unmodifiable(blocked);

  final ProfileSummary viewer;
  final ProfileId targetId;
  final Set<ProfileId> followed;
  final Set<ProfileId> blocked;
}

class ProfileDetailsPolicy {
  const ProfileDetailsPolicy();

  ProfileDetails build(
    ProfileSocialContext context,
    List<VideoPost> posts,
  ) {
    final items =
        posts.where((post) => post.creator.id == context.targetId).toList();
    final isCurrentUser = context.targetId == context.viewer.id;
    return ProfileDetails(
      profile: _summary(context, items),
      posts: items,
      statistics: _statistics(context, items, isCurrentUser),
      relationship: _relationship(context, isCurrentUser),
    );
  }

  ProfileSummary _summary(
    ProfileSocialContext context,
    List<VideoPost> posts,
  ) {
    if (posts.isNotEmpty) return posts.first.creator;
    return context.targetId == context.viewer.id
        ? context.viewer
        : ProfileSummary.unknown(context.targetId);
  }

  ProfileStatistics _statistics(
    ProfileSocialContext context,
    List<VideoPost> posts,
    bool isCurrentUser,
  ) {
    return ProfileStatistics(
      totalLikes: posts.fold(0, (sum, post) => sum + post.likeCount),
      followingCount: isCurrentUser ? context.followed.length : 0,
    );
  }

  ProfileRelationship _relationship(
    ProfileSocialContext context,
    bool isCurrentUser,
  ) {
    return ProfileRelationship(
      isFollowing:
          !isCurrentUser && context.followed.contains(context.targetId),
      isBlocked: !isCurrentUser && context.blocked.contains(context.targetId),
      isCurrentUser: isCurrentUser,
    );
  }
}
