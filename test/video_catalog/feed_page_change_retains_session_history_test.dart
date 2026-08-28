import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('leaving a video retains it only in bounded session history', () async {
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
    expect(loaded.posts.map((post) => post.id.value), [
      'first',
      'second',
      'third',
    ]);
    expect(loaded.roster.active.id.value, 'second');
    expect(history.entries.map((entry) => entry.videoId), [
      'e:second',
      'e:first',
    ]);
  });
}
