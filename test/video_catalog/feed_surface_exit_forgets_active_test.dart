import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'returning to an ordinary feed cannot restore its exited video',
    () async {
      final history = FakeWatchHistoryRepository();
      final source = FakeVideoCatalogRepository(
        forYouFeed: [
          samplePost(id: 'first'),
          samplePost(id: 'second'),
        ],
      );
      final feed = WatchAwareVideoFeedRepository(
        feed: source,
        history: history,
        failureReporter: RecordingFailureReporter(),
      );
      final cubit = FeedCubit(
        FeedDependencies(
          feed: feed,
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

      cubit.surfaceVisibilityChanged(false);
      expect(cubit.state, isA<FeedLoading>());
      await cubit.refresh();

      final returned = cubit.state as FeedLoaded;
      expect(returned.posts.first.id.value, 'second');
    },
  );
}
