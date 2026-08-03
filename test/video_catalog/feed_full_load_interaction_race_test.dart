import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/sample_data.dart';

void main() {
  test('a like accepted during a full load survives its stale result',
      () async {
    final original = samplePost();
    final repository = _PendingLoadRepository(original);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();

    final load = cubit.load(FeedKind.following);
    await cubit.toggleLike(original);
    repository.pending.complete([original]);
    await load;

    final post = (cubit.state as FeedLoaded).posts.single;
    expect(post.viewerHasLiked, isTrue);
    expect(post.likeCount, original.likeCount + 1);
  });
}

class _PendingLoadRepository
    implements VideoFeedRepository, VideoEngagementRepository {
  _PendingLoadRepository(this.original);

  final VideoPost original;
  final pending = Completer<List<VideoPost>>();
  var loads = 0;

  @override
  Future<List<VideoPost>> loadFeed(FeedKind kind,
      {bool excludeWatched = false}) {
    return loads++ == 0 ? Future.value([original]) : pending.future;
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async {
    return VideoFeedPage(posts: const <VideoPost>[]);
  }

  @override
  Future<VideoPost> toggleLike(VideoPost post) async {
    return post.withInteraction(
      VideoInteractionUpdate(
        likeCount: post.likeCount + 1,
        viewerHasLiked: true,
      ),
    );
  }
}
