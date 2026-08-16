import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a missing passive row keeps its original watch and position', () async {
    final history = FakeWatchHistoryRepository();
    final first = samplePost(id: 'first');
    final second = samplePost(id: 'second');
    final source = FakeVideoCatalogRepository(forYouFeed: [first, second]);
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
    source.forYouFeed.remove(first);

    await cubit.refresh();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts.map((post) => post.id.value), ['first', 'second']);
    expect(loaded.activeIndex, 0);
    expect(history.entries, hasLength(1));
    expect(history.entries.single.videoId, 'e:first');
  });
}
