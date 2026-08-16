import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_refresh_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('refresh survives an interaction during a pending watch', () async {
    final history = _SecondWatchGatedHistory();
    final source = _GatedRefreshSource();
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
      if (!source.release.isCompleted) source.release.complete();
      if (!history.release.isCompleted) history.release.complete();
      await cubit.close();
    });
    await cubit.load();
    cubit.pageChanged(1);
    await history.secondStarted.future;

    final refresh = cubit.refresh();
    await source.started.future;
    source.release.complete();
    await pumpEventQueue();
    cubit.commentsPublished(source.second, 1);
    history.release.complete();
    await refresh;

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts.map((post) => post.id.value), [
      'first',
      'second',
      'new',
    ]);
    expect(loaded.posts[1].commentCount, source.second.commentCount + 1);
  });
}

final class _GatedRefreshSource extends FakeVideoCatalogRepository
    implements VideoFeedRefreshRepository {
  _GatedRefreshSource()
    : first = samplePost(id: 'first'),
      second = samplePost(id: 'second'),
      fresh = samplePost(id: 'new'),
      super(forYouFeed: []) {
    forYouFeed.addAll([first, second]);
  }

  final started = Completer<void>();
  final release = Completer<void>();
  final VideoPost first;
  final VideoPost second;
  final VideoPost fresh;

  @override
  Future<VideoFeedRefreshSnapshot> loadRefresh(FeedKind kind) async {
    started.complete();
    await release.future;
    return VideoFeedRefreshSnapshot(
      allPosts: [first, second, fresh],
      eligiblePosts: [first, second, fresh],
    );
  }
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
