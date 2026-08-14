import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/filtered_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/following_feed_scope.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_reader.dart';

import '../support/fake_social_graph_repository.dart';
import '../support/following_feed_scope_fixture.dart';
import '../support/repost_samples.dart';

void main() {
  test('following loads use the repost-aware reader capability', () async {
    final followed = repostedPost().repost!.reposter.id;
    final reader = _FollowingReader(repostedPost());
    final social = FakeSocialGraphRepository(followed: {followed});
    final feed = FilteredVideoFeedRepository(
      reader,
      social,
      followingScopes: testFollowingFeedScopes(social),
    );

    final initial = await feed.loadFeed(FeedKind.following);
    final older = await feed.loadOlderFeed(
      FeedKind.following,
      olderThan: DateTime.utc(2026, 3),
    );

    expect(initial, hasLength(1));
    expect(older.posts, hasLength(1));
    expect(reader.followingLoads, 2);
  });
}

final class _FollowingReader
    implements VideoPostReader, FollowingVideoPostReader {
  _FollowingReader(this.post);
  final VideoPost post;
  var followingLoads = 0;

  @override
  Future<List<VideoPost>> loadFollowing(FollowingFeedScope scope) async {
    followingLoads += 1;
    return [post];
  }

  @override
  Future<List<VideoPost>> loadOlderFollowing({
    required DateTime olderThan,
    required FollowingFeedScope scope,
  }) async {
    followingLoads += 1;
    return [post];
  }

  @override
  Future<List<VideoPost>> load({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) => throw StateError('Generic following load used');

  @override
  Future<List<VideoPost>> loadOlder({
    required DateTime olderThan,
    Set<ProfileId>? creatorIds,
  }) => throw StateError('Generic following page used');
}
