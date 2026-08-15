import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('the feed closes when a newly visible watch cannot persist', () async {
    final posts = [samplePost(id: 'first'), samplePost(id: 'second')];
    final source = FakeVideoCatalogRepository(forYouFeed: posts);
    final cubit = FeedCubit(
      FeedDependencies(
        feed: source,
        engagement: source,
        optional: FeedOptionalDependencies(
          watch: FeedWatchDependencies(
            tracker: WatchHistoryTracker(
              history: _SecondWriteFailsHistory(),
              failureReporter: RecordingFailureReporter(),
            ),
          ),
        ),
      ),
    );
    addTearDown(cubit.close);
    await cubit.load();
    await pumpEventQueue();

    cubit.pageChanged(1);
    await pumpEventQueue();

    expect(cubit.state, isA<FeedFailure>());
  });
}

final class _SecondWriteFailsHistory extends FakeWatchHistoryRepository {
  var _writes = 0;

  @override
  Future<void> record(WatchHistoryEntry entry) async {
    _writes += 1;
    if (_writes == 2) throw const AppFailure('Storage unavailable.');
    await super.record(entry);
  }
}
