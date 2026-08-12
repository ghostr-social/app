import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class ProfileStatistics {
  factory ProfileStatistics({
    required int totalLikes,
    required int followingCount,
  }) {
    _checkCount(totalLikes, 'totalLikes');
    _checkCount(followingCount, 'followingCount');
    return ProfileStatistics._(totalLikes, followingCount);
  }

  const ProfileStatistics._(this.totalLikes, this.followingCount);

  final int totalLikes;
  final int followingCount;
}

class ProfileRelationship {
  factory ProfileRelationship({
    required bool isFollowing,
    required bool isBlocked,
    required bool isCurrentUser,
  }) {
    if (isCurrentUser && (isFollowing || isBlocked)) {
      throw StateError('A current-user profile cannot be followed or blocked.');
    }
    return ProfileRelationship._(isFollowing, isBlocked, isCurrentUser);
  }

  const ProfileRelationship._(
    this.isFollowing,
    this.isBlocked,
    this.isCurrentUser,
  );

  final bool isFollowing;
  final bool isBlocked;
  final bool isCurrentUser;
}

class ProfileDetails {
  factory ProfileDetails({
    required ProfileSummary profile,
    required List<VideoPost> posts,
    required ProfileStatistics statistics,
    required ProfileRelationship relationship,
  }) {
    return ProfileDetails._(
      profile,
      List<VideoPost>.unmodifiable(posts),
      statistics,
      relationship,
    );
  }

  const ProfileDetails._(
    this.profile,
    this.posts,
    this.statistics,
    this.relationship,
  );

  factory ProfileDetails.empty(ProfileSummary profile) {
    return ProfileDetails(
      profile: profile,
      posts: const <VideoPost>[],
      statistics: ProfileStatistics(totalLikes: 0, followingCount: 0),
      relationship: ProfileRelationship(
        isFollowing: false,
        isBlocked: false,
        isCurrentUser: false,
      ),
    );
  }

  final ProfileSummary profile;
  final List<VideoPost> posts;
  final ProfileStatistics statistics;
  final ProfileRelationship relationship;

  int get totalLikes => statistics.totalLikes;
  int get followingCount => statistics.followingCount;
  bool get isFollowing => relationship.isFollowing;
  bool get isBlocked => relationship.isBlocked;
  bool get isCurrentUser => relationship.isCurrentUser;

  ProfileDetails copyWith({bool? isFollowing, bool? isBlocked}) {
    return ProfileDetails(
      profile: profile,
      posts: posts,
      statistics: statistics,
      relationship: ProfileRelationship(
        isFollowing: isFollowing ?? this.isFollowing,
        isBlocked: isBlocked ?? this.isBlocked,
        isCurrentUser: isCurrentUser,
      ),
    );
  }

  ProfileDetails withProfile(ProfileSummary updated) {
    if (updated.id != profile.id) {
      throw StateError('Updated profile identity does not match.');
    }
    return ProfileDetails(
      profile: updated,
      posts: posts,
      statistics: statistics,
      relationship: relationship,
    );
  }
}

void _checkCount(int count, String name) {
  if (count < 0) throw RangeError.value(count, name, 'Cannot be negative.');
}
