import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/following_feed_scope.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import 'fake_remote_video_source.dart';

final class FakeFollowingRemoteVideoSource extends FakeRemoteVideoSource
    implements FollowingRemoteVideoSource, FollowingRemoteVideoUpdates {
  FakeFollowingRemoteVideoSource({
    this.followingPosts = const <VideoPost>[],
    this.followingOlderPosts = const <VideoPost>[],
  }) : super(const <VideoPost>[]);

  final List<VideoPost> followingPosts;
  final List<VideoPost> followingOlderPosts;
  AppFailure? followingFailure;
  FollowingFeedScope? requestedFollowingScope;
  DateTime? requestedFollowingOlderThan;
  int followingWatchCount = 0;

  @override
  Future<List<VideoPost>> loadFollowingRemoteFeed(
    FollowingFeedScope scope, {
    DateTime? olderThan,
  }) async {
    requestedFollowingScope = scope;
    requestedFollowingOlderThan = olderThan;
    if (followingFailure case final failure?) throw failure;
    return olderThan == null ? followingPosts : followingOlderPosts;
  }

  @override
  Stream<RemoteVideoSnapshot> watchFollowingRemoteFeed(
    FollowingFeedScope scope,
  ) {
    requestedFollowingScope = scope;
    followingWatchCount += 1;
    return Stream<RemoteVideoSnapshot>.value(
      RemoteVideoSnapshot(
        revision: BigInt.one,
        phase: snapshotPhase,
        posts: followingPosts,
      ),
    );
  }
}
