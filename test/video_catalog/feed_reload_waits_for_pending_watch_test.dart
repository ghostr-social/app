import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a reload waits for a pending watch before filtering', () async {
    final history = _GatedWatchHistory();
    final fresh = List.generate(10, (index) => samplePost(id: 'fresh-$index'));
    final source = FakeVideoCatalogRepository(
      forYouFeed: [
        samplePost(id: 'first'),
        samplePost(id: 'pending'),
        ...fresh,
      ],
    );
    final reporter = RecordingFailureReporter();
    final cubit = FeedCubit(
      FeedDependencies(
        feed: WatchAwareVideoFeedRepository(
          feed: source,
          history: history,
          failureReporter: reporter,
        ),
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
    addTearDown(cubit.close);
    addTearDown(() {
      if (!history.releasePendingWrite.isCompleted) {
        history.releasePendingWrite.complete();
      }
    });
    await cubit.load();
    expect(history._filters, 1);

    cubit.pageChanged(1);
    await history.pendingWriteStarted.future;
    expect((cubit.state as FeedLoaded).roster.active.id.value, 'pending');
    expect(history._filters, 1);
    final reload = cubit.reload();
    await pumpEventQueue();
    expect(history.reloadFiltered.isCompleted, isFalse);
    history.releasePendingWrite.complete();
    await reload;

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts.map((post) => post.id), fresh.map((post) => post.id));
  });
}

final class _GatedWatchHistory extends FakeWatchHistoryRepository {
  final pendingWriteStarted = Completer<void>();
  final releasePendingWrite = Completer<void>();
  final reloadFiltered = Completer<void>();
  var _writes = 0;
  var _filters = 0;

  @override
  Future<void> record(WatchHistoryEntry entry) async {
    _writes += 1;
    if (_writes == 2) {
      pendingWriteStarted.complete();
      await releasePendingWrite.future;
    }
    await super.record(entry);
  }

  @override
  Future<List<VideoPost>> filterUnwatched(List<VideoPost> posts) async {
    final filtered = await super.filterUnwatched(posts);
    _filters += 1;
    if (_filters == 2) reloadFiltered.complete();
    return filtered;
  }
}
