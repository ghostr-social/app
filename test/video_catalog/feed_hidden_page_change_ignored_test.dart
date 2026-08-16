import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a page callback after hiding cannot move the feed', () async {
    final history = FakeWatchHistoryRepository();
    final posts = [samplePost(id: 'first'), samplePost(id: 'second')];
    final source = FakeVideoCatalogRepository(forYouFeed: posts);
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

    cubit.surfaceVisibilityChanged(false);
    expect(cubit.pageChanged(1), isFalse);

    final hidden = cubit.state as FeedLoaded;
    expect(hidden.activeIndex, 0);
    expect(history.entries.single.videoId, 'e:first');
  });
}
