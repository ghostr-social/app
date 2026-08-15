import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('leaving a video removes it from the ordinary feed roster', () async {
    final posts = [
      samplePost(id: 'first'),
      samplePost(id: 'second'),
      samplePost(id: 'third'),
    ];
    final history = FakeWatchHistoryRepository();
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

    cubit.pageChanged(1);
    await pumpEventQueue();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts.map((post) => post.id.value), ['second', 'third']);
    expect(loaded.activeIndex, 0);
    expect(history.entries.map((entry) => entry.videoId), [
      'e:second',
      'e:first',
    ]);
  });
}
