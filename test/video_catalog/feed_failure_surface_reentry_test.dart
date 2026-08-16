import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a failed feed retries when its surface returns', () async {
    final source = _FailingOnceFeed();
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
    await cubit.load();
    expect(cubit.state, isA<FeedFailure>());

    cubit.surfaceVisibilityChanged(false);
    cubit.surfaceVisibilityChanged(true);
    await pumpEventQueue();

    expect(source.loads, 2);
    expect(cubit.state, isA<FeedLoaded>());
  });
}

final class _FailingOnceFeed extends FakeVideoCatalogRepository {
  _FailingOnceFeed() : super(forYouFeed: [samplePost()]);

  var loads = 0;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    loads += 1;
    if (loads == 1) throw const AppFailure('Feed failed.');
    return super.loadFeed(kind, excludeWatched: excludeWatched);
  }
}
