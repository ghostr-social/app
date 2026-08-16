import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_refresh_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'a swipe supersedes refresh eligibility captured before its watch',
    () async {
      final posts = List.generate(3, (index) => samplePost(id: 'p$index'));
      final source = _GatedRefreshSource(posts);
      final history = FakeWatchHistoryRepository();
      final cubit = FeedCubit(
        FeedDependencies(
          feed: source,
          engagement: source,
          optional: FeedOptionalDependencies(
            watch: FeedWatchDependencies(
              tracker: WatchHistoryTracker(
                history: history,
                failureReporter: RecordingFailureReporter(),
              ),
            ),
          ),
        ),
      );
      addTearDown(cubit.close);
      await cubit.load();
      final refresh = cubit.refresh();
      await source.refreshStarted.future;

      cubit.pageChanged(1);
      await pumpEventQueue();
      source.releaseRefresh.complete();
      await refresh;

      final loaded = cubit.state as FeedLoaded;
      expect(loaded.posts.map((post) => post.id.value), ['p0', 'p1', 'p2']);
      expect(loaded.activeIndex, 1);
    },
  );
}

final class _GatedRefreshSource extends FakeVideoCatalogRepository
    implements VideoFeedRefreshRepository {
  _GatedRefreshSource(List<VideoPost> posts) : super(forYouFeed: posts);

  final refreshStarted = Completer<void>();
  final releaseRefresh = Completer<void>();

  @override
  Future<VideoFeedRefreshSnapshot> loadRefresh(FeedKind kind) async {
    refreshStarted.complete();
    await releaseRefresh.future;
    return VideoFeedRefreshSnapshot(
      allPosts: forYouFeed,
      eligiblePosts: forYouFeed.skip(1).toList(),
    );
  }
}
