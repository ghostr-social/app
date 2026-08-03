import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/sample_data.dart';

void main() {
  test('keeps a comment floor until an observed count reaches it', () async {
    final original = samplePost();
    final stale = _comments(original, original.commentCount, observed: true);
    final confirmed = _comments(original, 12, observed: true);
    final fallback = _comments(original, 0);
    final remoteDeletion = _comments(original, 11, observed: true);
    final repository = _Repository([
      original,
      stale,
      confirmed,
      fallback,
      remoteDeletion,
    ]);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();

    cubit.commentsPublished(original, 1);
    await cubit.refresh();
    expect(_post(cubit).commentCount, original.commentCount + 1);

    await cubit.refresh();
    expect(_post(cubit).commentCount, 12);
    await cubit.refresh();
    expect(_post(cubit).commentCount, 12);
    await cubit.refresh();
    expect(_post(cubit).commentCount, 11);
  });
}

VideoPost _post(FeedCubit cubit) => (cubit.state as FeedLoaded).posts.single;

VideoPost _comments(VideoPost post, int count, {bool observed = false}) {
  return post.withInteraction(
    VideoInteractionUpdate(
      likeCount: post.likeCount,
      viewerHasLiked: post.viewerHasLiked,
      commentCount: count,
      observations: VideoMetricObservationUpdate(
        comments: observed
            ? VideoMetricObservation.observed
            : VideoMetricObservation.unobserved,
      ),
    ),
  );
}

class _Repository implements VideoFeedRepository, VideoEngagementRepository {
  _Repository(this.results);

  final List<VideoPost> results;
  var _load = 0;

  @override
  Future<List<VideoPost>> loadFeed(FeedKind kind) async => [results[_load++]];

  @override
  Future<VideoPost> toggleLike(VideoPost post) => throw UnimplementedError();
}
