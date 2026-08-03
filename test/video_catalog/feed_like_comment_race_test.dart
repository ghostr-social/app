import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/sample_data.dart';

void main() {
  test('a delayed like preserves a newly published comment count', () async {
    final post = samplePost();
    final repository = _DelayedLikeRepository(post);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();

    final like = cubit.toggleLike(post);
    cubit.commentsPublished(post, 1);
    repository.like.complete(post.withInteraction(
      VideoInteractionUpdate(
        likeCount: post.likeCount + 1,
        viewerHasLiked: true,
      ),
    ));
    await like;

    final updated = (cubit.state as FeedLoaded).posts.single;
    expect(updated.commentCount, post.commentCount + 1);
    expect(updated.likeCount, post.likeCount + 1);
    expect(updated.viewerHasLiked, isTrue);
  });
}

class _DelayedLikeRepository
    implements VideoFeedRepository, VideoEngagementRepository {
  _DelayedLikeRepository(this.post);

  final VideoPost post;
  final like = Completer<VideoPost>();

  @override
  Future<List<VideoPost>> loadFeed(FeedKind kind,
          {bool excludeWatched = false}) async =>
      <VideoPost>[post];

  @override
  Future<VideoPost> toggleLike(VideoPost post) => like.future;
}
