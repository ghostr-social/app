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
  test('a new feed load supersedes refresh waiting on a swipe', () async {
    final first = samplePost(id: 'first');
    final second = samplePost(id: 'second');
    final following = samplePost(id: 'following');
    final source = _GatedFollowingSource([first, second], following);
    final history = _SecondWatchGatedHistory();
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
      if (!source.release.isCompleted) source.release.complete();
      await cubit.close();
    });
    await cubit.load();

    cubit.pageChanged(1);
    await history.secondStarted.future;
    final refresh = cubit.refresh();
    await pumpEventQueue();
    final loading = cubit.load(FeedKind.following);
    await pumpEventQueue();

    expect(cubit.state, isA<FeedLoading>());
    expect(cubit.state.kind, FeedKind.following);
    history.release.complete();
    await refresh;
    expect(cubit.state, isA<FeedLoading>());
    source.release.complete();
    await loading;
    final loaded = cubit.state as FeedLoaded;
    expect(loaded.kind, FeedKind.following);
    expect(loaded.roster.active.id, following.id);
  });
}

final class _GatedFollowingSource extends FakeVideoCatalogRepository
    implements VideoFeedRefreshRepository {
  _GatedFollowingSource(List<VideoPost> forYouFeed, this.following)
    : super(forYouFeed: forYouFeed);
  final VideoPost following;
  final release = Completer<void>();

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    if (kind == FeedKind.forYou) return forYouFeed;
    await release.future;
    return [following];
  }

  @override
  Future<VideoFeedRefreshSnapshot> loadRefresh(FeedKind kind) async {
    return VideoFeedRefreshSnapshot(
      allPosts: forYouFeed,
      eligiblePosts: forYouFeed,
    );
  }
}

final class _SecondWatchGatedHistory extends FakeWatchHistoryRepository {
  final secondStarted = Completer<void>();
  final release = Completer<void>();
  var writes = 0;

  @override
  Future<void> record(WatchHistoryEntry entry) async {
    if (++writes != 2) return super.record(entry);
    secondStarted.complete();
    await release.future;
    await super.record(entry);
  }
}
