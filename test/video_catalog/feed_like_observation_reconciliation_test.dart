import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/sample_data.dart';

void main() {
  test('keeps an accepted like until an observed refresh confirms it',
      () async {
    final original = samplePost();
    final stale = _like(original, original.likeCount, false, observed: true);
    final confirmed = _like(original, 50, true, observed: true);
    final fallback = _like(original, 0, false);
    final remoteUnlike = _like(original, 49, false, observed: true);
    final repository = _Repository([
      original,
      stale,
      confirmed,
      fallback,
      remoteUnlike,
    ]);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();

    await cubit.toggleLike(original);
    await cubit.refresh();
    expect(_post(cubit).viewerHasLiked, isTrue);
    expect(_post(cubit).likeCount, original.likeCount + 1);

    await cubit.refresh();
    expect(_post(cubit).likeCount, 50);
    await cubit.refresh();
    expect(_post(cubit).likeCount, 50);
    await cubit.refresh();
    expect(_post(cubit).viewerHasLiked, isFalse);
    expect(_post(cubit).likeCount, 49);
  });
}

VideoPost _post(FeedCubit cubit) => (cubit.state as FeedLoaded).posts.single;

VideoPost _like(VideoPost post, int count, bool liked,
    {bool observed = false}) {
  return post.withInteraction(
    VideoInteractionUpdate(
      likeCount: count,
      viewerHasLiked: liked,
      observations: VideoMetricObservationUpdate(
        likes: observed
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
  Future<VideoPost> toggleLike(VideoPost post) async {
    return _like(post, post.likeCount + 1, true);
  }
}
