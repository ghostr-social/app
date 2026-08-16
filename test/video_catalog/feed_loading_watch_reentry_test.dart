import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a hidden watch commit keeps its video for the pending feed', () async {
    final history = _FirstWatchGatedHistory();
    final posts = [samplePost(id: 'first'), samplePost(id: 'second')];
    final source = FakeVideoCatalogRepository(forYouFeed: posts);
    final reporter = RecordingFailureReporter();
    final feed = WatchAwareVideoFeedRepository(
      feed: source,
      history: history,
      failureReporter: reporter,
    );
    final cubit = FeedCubit(
      FeedDependencies(
        feed: feed,
        engagement: source,
        optional: FeedOptionalDependencies(
          watch: FeedWatchDependencies(
            tracker: WatchHistoryTracker(
              history: history,
              failureReporter: reporter,
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

    cubit.surfaceVisibilityChanged(false);
    history.release.complete();
    await loading;
    cubit.surfaceVisibilityChanged(true);
    await pumpEventQueue();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts.map((post) => post.id.value), ['first', 'second']);
    expect(loaded.activeIndex, 0);
    expect(history.entries.single.videoId, 'e:first');
  });
}

final class _FirstWatchGatedHistory extends FakeWatchHistoryRepository {
  final started = Completer<void>();
  final release = Completer<void>();

  @override
  Future<void> record(WatchHistoryEntry entry) async {
    if (!started.isCompleted) started.complete();
    await release.future;
    await super.record(entry);
  }
}
