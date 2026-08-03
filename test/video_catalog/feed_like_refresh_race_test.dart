import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/sample_data.dart';

void main() {
  test('a delayed like preserves a newer feed snapshot', () async {
    final original = samplePost();
    final refreshed = samplePost(caption: 'Fresh caption').withInteraction(
      VideoInteractionUpdate(
        likeCount: original.likeCount,
        commentCount: 20,
        viewerHasLiked: false,
        observations: const VideoMetricObservationUpdate(
          likes: VideoMetricObservation.observed,
          comments: VideoMetricObservation.observed,
        ),
      ),
    );
    final repository = _RefreshDuringLikeRepository(original, refreshed);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();

    final like = cubit.toggleLike(original);
    await cubit.refresh();
    repository.like.complete(original.withInteraction(
      VideoInteractionUpdate(
        likeCount: original.likeCount + 1,
        viewerHasLiked: true,
      ),
    ));
    await like;

    final updated = (cubit.state as FeedLoaded).posts.single;
    expect(updated.caption, refreshed.caption);
    expect(updated.commentCount, refreshed.commentCount);
    expect(updated.likeCount, original.likeCount + 1);
  });
}

class _RefreshDuringLikeRepository
    implements VideoFeedRepository, VideoEngagementRepository {
  _RefreshDuringLikeRepository(this.original, this.refreshed);

  final VideoPost original;
  final VideoPost refreshed;
  final like = Completer<VideoPost>();
  var loads = 0;

  @override
  Future<List<VideoPost>> loadFeed(FeedKind kind,
      {bool excludeWatched = false}) async {
    return <VideoPost>[loads++ == 0 ? original : refreshed];
  }

  @override
  Future<VideoPost> toggleLike(VideoPost post) => like.future;
}
