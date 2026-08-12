import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/social/domain/follow_outcome.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/creator_search_source.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class RecordingRemoteVideoSource implements RemoteVideoSource {
  RecordingRemoteVideoSource(this.posts);

  final List<VideoPost> posts;
  final List<String?> searchQueries = <String?>[];
  final List<Set<String>?> hashtags = <Set<String>?>[];
  final List<DateTime?> olderThans = <DateTime?>[];
  final List<String?> loadMoreQueries = <String?>[];
  final List<Set<String>?> loadMoreHashtags = <Set<String>?>[];
  final List<String?> watchQueries = <String?>[];
  final List<Set<String>?> watchHashtags = <Set<String>?>[];
  RemoteVideoPhase snapshotPhase = RemoteVideoPhase.settled;

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
    DateTime? olderThan,
  }) async {
    searchQueries.add(searchQuery);
    this.hashtags.add(hashtags);
    olderThans.add(olderThan);
    return posts;
  }

  @override
  Future<List<VideoPost>> loadMoreRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) async {
    loadMoreQueries.add(searchQuery);
    loadMoreHashtags.add(hashtags);
    return posts;
  }

  @override
  Stream<RemoteVideoSnapshot> watchRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) {
    watchQueries.add(searchQuery);
    watchHashtags.add(hashtags);
    return Stream.value(
      RemoteVideoSnapshot(
        revision: BigInt.one,
        phase: snapshotPhase,
        posts: posts,
      ),
    );
  }
}

class RecordingCreatorSearchSource implements CreatorSearchSource {
  RecordingCreatorSearchSource(this.creators);

  final List<ProfileSummary> creators;
  final List<String> queries = <String>[];
  AppFailure? failure;

  @override
  Future<List<ProfileSummary>> searchCreators(String query) async {
    queries.add(query);
    if (failure case final AppFailure error) throw error;
    return creators;
  }
}

class FakeSocialGraph implements SocialGraphRepository {
  final Set<ProfileId> blocked = <ProfileId>{};
  final Set<ProfileId> followed = <ProfileId>{};
  @override
  Future<Set<ProfileId>> loadBlockedProfiles() async => blocked;
  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async => followed;
  @override
  Future<FollowOutcome> follow(ProfileId profileId) async {
    return followed.add(profileId)
        ? FollowOutcome.newlyFollowed
        : FollowOutcome.alreadyFollowing;
  }

  @override
  Future<bool> toggleBlock(ProfileId profileId) async => blocked.add(profileId);
  @override
  Future<bool> toggleFollow(ProfileId profileId) async =>
      followed.remove(profileId) ? false : followed.add(profileId);
}
