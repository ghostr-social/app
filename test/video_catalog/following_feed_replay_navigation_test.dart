import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('Following keeps prior videos reachable for explicit replay', () async {
    final posts = [samplePost(id: 'first'), samplePost(id: 'second')];
    final source = FakeVideoCatalogRepository(
      forYouFeed: const [],
      feed: FakeFeedScenario(followingFeed: posts),
    );
    final cubit = FeedCubit(
      FeedDependencies(
        feed: source,
        engagement: source,
        optional: FeedOptionalDependencies(
          watch: FeedWatchDependencies(
            tracker: WatchHistoryTracker(
              history: FakeWatchHistoryRepository(),
              failureReporter: RecordingFailureReporter(),
            ),
          ),
        ),
      ),
    );
    addTearDown(cubit.close);
    await cubit.load(FeedKind.following);

    cubit.pageChanged(1);
    await pumpEventQueue();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts.map((post) => post.id.value), ['first', 'second']);
    expect(loaded.activeIndex, 1);
  });
}
