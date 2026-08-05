import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_updates.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class FakeRemoteVideoSource implements RemoteVideoSource {
  FakeRemoteVideoSource(this.posts);

  final List<VideoPost> posts;
  List<VideoPost> olderPosts = const <VideoPost>[];
  AppFailure? failure;
  RemoteVideoPhase snapshotPhase = RemoteVideoPhase.settled;
  int loadCount = 0;
  Set<ProfileId>? requestedCreatorIds;
  String? requestedSearchQuery;
  Set<String>? requestedHashtags;
  Set<ProfileId>? requestedLoadMoreCreatorIds;
  String? requestedLoadMoreSearchQuery;
  Set<String>? requestedLoadMoreHashtags;
  final List<DateTime> requestedOlderThan = <DateTime>[];

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
    DateTime? olderThan,
  }) async {
    loadCount += 1;
    requestedCreatorIds = creatorIds;
    requestedSearchQuery = searchQuery;
    requestedHashtags = hashtags;
    if (olderThan != null) requestedOlderThan.add(olderThan);
    if (failure case final failure?) throw failure;
    return olderThan == null ? posts : olderPosts;
  }

  @override
  Future<List<VideoPost>> loadMoreRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) async {
    loadCount += 1;
    requestedLoadMoreCreatorIds = creatorIds;
    requestedLoadMoreSearchQuery = searchQuery;
    requestedLoadMoreHashtags = hashtags;
    return olderPosts;
  }

  @override
  Stream<RemoteVideoSnapshot> watchRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) {
    return Stream.value(RemoteVideoSnapshot(
      revision: BigInt.one,
      phase: snapshotPhase,
      posts: posts,
    ));
  }
}
