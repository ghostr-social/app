import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('duplicate page changes wait for the same durable watch', () async {
    final history = _SecondWatchGatedHistory();
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
    addTearDown(() async {
      if (!history.release.isCompleted) history.release.complete();
      await cubit.close();
    });
    await cubit.load();

    cubit.pageChanged(1);
    await history.secondStarted.future;
    cubit.pageChanged(1);

    expect((cubit.state as FeedLoaded).roster.active.id.value, 'first');
    history.release.complete();
    await pumpEventQueue();
    expect((cubit.state as FeedLoaded).roster.active.id.value, 'second');
  });
}

final class _SecondWatchGatedHistory extends FakeWatchHistoryRepository {
  final secondStarted = Completer<void>();
  final release = Completer<void>();
  var writes = 0;

  @override
  Future<void> record(WatchHistoryEntry entry) async {
    writes += 1;
    if (writes == 2) {
      secondStarted.complete();
      await release.future;
    }
    await super.record(entry);
  }
}
