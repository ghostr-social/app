import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a rapid reversal supersedes the pending forward swipe', () async {
    final history = _GatedHistory();
    final source = FakeVideoCatalogRepository(
      forYouFeed: [
        samplePost(id: 'a'),
        samplePost(id: 'b'),
      ],
    );
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
    addTearDown(() async {
      if (!history.release.isCompleted) history.release.complete();
      await cubit.close();
    });
    await cubit.load();

    cubit.pageChanged(1);
    await history.secondStarted.future;
    cubit.pageChanged(0);
    history.release.complete();
    await pumpEventQueue();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts[loaded.activeIndex].id.value, 'a');
  });
}

final class _GatedHistory extends FakeWatchHistoryRepository {
  final secondStarted = Completer<void>();
  final release = Completer<void>();
  var writes = 0;

  @override
  Future<void> record(WatchHistoryEntry entry) async {
    if (++writes == 2) {
      secondStarted.complete();
      await release.future;
    }
    await super.record(entry);
  }
}
