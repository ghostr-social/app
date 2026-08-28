import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'refresh preserves the visited forward branch after a back swipe',
    () async {
      final posts = List.generate(7, (index) => samplePost(id: 'post-$index'));
      final source = FakeVideoCatalogRepository(forYouFeed: posts);
      final history = FakeWatchHistoryRepository();
      final reporter = RecordingFailureReporter();
      final feed = WatchAwareVideoFeedRepository(
        feed: source,
        history: history,
        failureReporter: reporter,
      );
      final cubit = FeedCubit(
        FeedDependencies(
          feed: feed,
          engagement: source,
          optional: FeedOptionalDependencies(
            watch: FeedWatchDependencies(
              tracker: WatchHistoryTracker(
                history: history,
                failureReporter: reporter,
              ),
            ),
          ),
        ),
      );
      addTearDown(cubit.close);
      await cubit.load();
      for (var swipe = 0; swipe < 4; swipe += 1) {
        final loaded = cubit.state as FeedLoaded;
        cubit.pageChanged(loaded.activeIndex + 1);
        await pumpEventQueue();
      }
      var loaded = cubit.state as FeedLoaded;
      cubit.pageChanged(loaded.activeIndex - 1);
      await pumpEventQueue();
      await history.record(
        WatchHistoryEntry.fromPost(posts[5], DateTime.utc(2026, 8, 27)),
      );

      await cubit.refresh();

      loaded = cubit.state as FeedLoaded;
      expect(loaded.posts.map((post) => post.id.value), [
        'post-1',
        'post-2',
        'post-3',
        'post-4',
        'post-6',
      ]);
      expect(loaded.roster.active.id.value, 'post-3');
      expect(loaded.posts[loaded.activeIndex + 1].id.value, 'post-4');
    },
  );
}
