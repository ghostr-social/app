import 'dart:async';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_refresh_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';
import '../support/controllable_video_feed_updates.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a live revision supersedes refresh waiting on a swipe', () async {
    final first = samplePost(id: 'first');
    final second = samplePost(id: 'second');
    final source = _RevisionSource([first, second]);
    final updates = ControllableVideoFeedUpdates();
    final history = SecondWriteGatedWatchHistoryRepository();
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
          delivery: FeedDeliveryDependencies(updates: updates),
        ),
      ),
    );
    addTearDown(() async {
      if (!history.release.isCompleted) history.release.complete();
      if (!source.releaseLive.isCompleted) source.releaseLive.complete();
      await cubit.close();
      await updates.close();
    });
    await cubit.load();
    cubit.pageChanged(1);
    await history.started.future;
    final refresh = cubit.refresh();
    await source.manualRead.future;
    updates.add(_update());
    await pumpEventQueue();
    history.release.complete();
    await source.liveRead.future;
    final pending = (cubit.state as FeedLoaded).posts;
    expect(pending.map((post) => post.id.value), isNot(contains('stale')));
    source.releaseLive.complete();
    await refresh;
    await pumpEventQueue();
    expect(
      (cubit.state as FeedLoaded).posts.map((post) => post.id.value),
      contains('fresh'),
    );
  });
}

VideoFeedUpdate _update() => VideoFeedUpdate(
  revision: BigInt.one,
  phase: VideoFeedUpdatePhase.loading,
  hasPosts: true,
);

final class _RevisionSource extends FakeVideoCatalogRepository
    implements VideoFeedRefreshRepository {
  _RevisionSource(List<VideoPost> posts) : super(forYouFeed: posts);
  final manualRead = Completer<void>();
  final liveRead = Completer<void>();
  final releaseLive = Completer<void>();
  var reads = 0;
  @override
  Future<VideoFeedRefreshSnapshot> loadRefresh(FeedKind kind) async {
    if (++reads == 1) {
      manualRead.complete();
      return _snapshot([forYouFeed.last, samplePost(id: 'stale')]);
    }
    liveRead.complete();
    await releaseLive.future;
    return _snapshot([forYouFeed.last, samplePost(id: 'fresh')]);
  }
}

VideoFeedRefreshSnapshot _snapshot(List<VideoPost> posts) =>
    VideoFeedRefreshSnapshot(allPosts: posts, eligiblePosts: posts);
