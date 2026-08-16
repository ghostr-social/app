import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/scripted_feed_repository.dart';

void main() {
  test('refresh applies a safe same-media revision to a watched row', () async {
    final initial = samplePost(id: 'stable', caption: 'Original caption');
    final revised = samplePost(id: 'stable', caption: 'Revised caption');
    final history = FakeWatchHistoryRepository();
    final source = ScriptedFeedRepository(
      loads: [
        [initial],
        [revised],
      ],
    );
    final reporter = RecordingFailureReporter();
    final feed = WatchAwareVideoFeedRepository(
      feed: source,
      history: history,
      failureReporter: reporter,
    );
    final cubit = FeedCubit(
      FeedDependencies(
        feed: feed,
        engagement: FakeVideoCatalogRepository(forYouFeed: const []),
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

    await cubit.refresh();

    final active = (cubit.state as FeedLoaded).roster.active;
    expect(active.id, revised.id);
    expect(active.caption, 'Revised caption');
    expect(history.entries, hasLength(1));
  });
}
