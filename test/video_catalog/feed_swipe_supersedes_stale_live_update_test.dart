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
  test('a swipe retries a superseded pending live update', () async {
    final initial = List.generate(2, (index) => samplePost(id: 'p$index'));
    final fresh = samplePost(id: 'p2');
    final source = _GatedLiveRefreshSource(initial, fresh);
    final updates = ControllableVideoFeedUpdates();
    final cubit = FeedCubit(
      FeedDependencies(
        feed: source,
        engagement: source,
        optional: FeedOptionalDependencies(
          watch: FeedWatchDependencies(
            tracker: WatchHistoryTracker(
              history: FakeWatchHistoryRepository(),
              failureReporter: RecordingFailureReporter(),
            ),
          ),
          delivery: FeedDeliveryDependencies(updates: updates),
        ),
      ),
    );
    addTearDown(() async {
      if (!source.releaseRefresh.isCompleted) source.releaseRefresh.complete();
      await cubit.close();
      await updates.close();
    });
    await cubit.load();
    updates.add(
      VideoFeedUpdate(
        revision: BigInt.one,
        phase: VideoFeedUpdatePhase.loading,
        hasPosts: true,
      ),
    );
    await source.refreshStarted.future;

    cubit.pageChanged(1);
    await pumpEventQueue();
    source.releaseRefresh.complete();
    await pumpEventQueue();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts.map((post) => post.id.value), ['p0', 'p1', 'p2']);
    expect(loaded.activeIndex, 1);
  });
}

final class _GatedLiveRefreshSource extends FakeVideoCatalogRepository
    implements VideoFeedRefreshRepository {
  _GatedLiveRefreshSource(List<VideoPost> posts, this.fresh)
    : super(forYouFeed: posts);

  final VideoPost fresh;
  final refreshStarted = Completer<void>();
  final releaseRefresh = Completer<void>();
  var refreshes = 0;

  @override
  Future<VideoFeedRefreshSnapshot> loadRefresh(FeedKind kind) async {
    refreshes += 1;
    if (refreshes == 1) {
      refreshStarted.complete();
      await releaseRefresh.future;
    }
    final posts = [...forYouFeed, fresh];
    return VideoFeedRefreshSnapshot(
      allPosts: posts,
      eligiblePosts: posts.skip(1).toList(),
    );
  }
}
