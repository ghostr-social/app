import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('refresh drops a held-ahead video watched on another surface', () async {
    final active = samplePost(id: 'active');
    final watchedAhead = samplePost(id: 'watched-ahead');
    final fresh = samplePost(id: 'fresh');
    final source = FakeVideoCatalogRepository(
      forYouFeed: [active, watchedAhead, fresh],
    );
    final history = FakeWatchHistoryRepository();
    final reporter = RecordingFailureReporter();
    final cubit = FeedCubit(
      FeedDependencies(
        feed: WatchAwareVideoFeedRepository(
          feed: source,
          history: history,
          failureReporter: reporter,
        ),
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
    await history.record(
      WatchHistoryEntry.fromPost(watchedAhead, DateTime.utc(2026, 8, 15)),
    );

    await cubit.refresh();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts.map((post) => post.id.value), ['active', 'fresh']);
    expect(loaded.activeIndex, 0);
  });
}
