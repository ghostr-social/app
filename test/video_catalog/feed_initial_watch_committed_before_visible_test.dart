import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('the initial video stays hidden until its watch is committed', () async {
    final history = _GatedWatchHistoryRepository();
    final source = FakeVideoCatalogRepository(forYouFeed: [samplePost()]);
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

    final loading = cubit.load();
    await history.started.future;

    expect(cubit.state, isA<FeedLoading>());
    history.release.complete();
    await loading;
    expect(cubit.state, isA<FeedLoaded>());
  });
}

final class _GatedWatchHistoryRepository extends FakeWatchHistoryRepository {
  final started = Completer<void>();
  final release = Completer<void>();

  @override
  Future<void> record(WatchHistoryEntry entry) async {
    started.complete();
    await release.future;
    await super.record(entry);
  }
}
