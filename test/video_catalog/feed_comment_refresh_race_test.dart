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
  test('refresh preserves comments published after it began', () async {
    final original = samplePost();
    final refreshed =
        samplePost(caption: 'Fresh canonical caption').withInteraction(
      VideoInteractionUpdate(
        likeCount: 70,
        commentCount: original.commentCount,
        viewerHasLiked: true,
        observations: const VideoMetricObservationUpdate(
          likes: VideoMetricObservation.observed,
          comments: VideoMetricObservation.observed,
        ),
      ),
    );
    final repository = _DelayedRefreshRepository(original);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();

    final refresh = cubit.refresh();
    cubit.commentsPublished(original, 1);
    repository.refresh.complete(<VideoPost>[refreshed]);
    await refresh;

    final post = (cubit.state as FeedLoaded).posts.single;
    expect(post.caption, 'Fresh canonical caption');
    expect(post.commentCount, original.commentCount + 1);
    expect(post.likeCount, 70);
    expect(post.viewerHasLiked, isTrue);
  });
}

class _DelayedRefreshRepository
    implements VideoFeedRepository, VideoEngagementRepository {
  _DelayedRefreshRepository(this.original);

  final VideoPost original;
  final refresh = Completer<List<VideoPost>>();
  var loads = 0;

  @override
  Future<List<VideoPost>> loadFeed(FeedKind kind,
      {bool excludeWatched = false}) {
    return loads++ == 0 ? Future.value(<VideoPost>[original]) : refresh.future;
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
  Future<VideoPost> toggleLike(VideoPost post) => throw UnimplementedError();
}
