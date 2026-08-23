import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_refresh_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/fake_feed_focus_port.dart';
import '../support/sample_data.dart';

void main() {
  test('refresh does not reverse a pending swipe it still contains', () async {
    final history = _SecondWatchGatedHistory();
    final focus = FakeFeedFocusPort();
    final posts = [samplePost(id: 'first'), samplePost(id: 'second')];
    final source = _RefreshSource(posts);
    final cubit = FeedCubit(
      FeedDependencies(
        feed: source,
        engagement: source,
        optional: FeedOptionalDependencies(
          focus: focus,
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
    final refresh = cubit.refresh();
    await pumpEventQueue();
    history.release.complete();
    await refresh;
    await pumpEventQueue();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.roster.active.id.value, 'second');
    expect(loaded.posts.map((post) => post.id.value), ['second']);
    expect(focus.focuses.skip(1).map((item) => item.cause), [
      FeedFocusCause.userNavigation,
      FeedFocusCause.rosterChange,
    ]);
    expect(focus.focuses.skip(1).map((item) => item.current.id.value), [
      'second',
      'second',
    ]);
  });
}

final class _RefreshSource extends FakeVideoCatalogRepository
    implements VideoFeedRefreshRepository {
  _RefreshSource(List<VideoPost> posts) : super(forYouFeed: posts);

  @override
  Future<VideoFeedRefreshSnapshot> loadRefresh(FeedKind kind) async {
    return VideoFeedRefreshSnapshot(
      allPosts: forYouFeed,
      eligiblePosts: [forYouFeed.last],
    );
  }
}

final class _SecondWatchGatedHistory extends FakeWatchHistoryRepository {
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
