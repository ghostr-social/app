import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_refresh_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/controllable_video_feed_updates.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('an update arriving during refresh is reconciled afterward', () async {
    final updates = ControllableVideoFeedUpdates();
    final feed = _GatedRefreshFeed();
    final cubit = FeedCubit(
      FeedDependencies(
        feed: feed,
        engagement: FakeVideoCatalogRepository(forYouFeed: const []),
        optional: FeedOptionalDependencies(
          delivery: FeedDeliveryDependencies(updates: updates),
        ),
      ),
    );
    addTearDown(updates.close);
    addTearDown(() async {
      if (!feed.release.isCompleted) feed.release.complete();
      await cubit.close();
    });
    await cubit.load();

    final refresh = cubit.refresh();
    await feed.firstStarted.future;
    updates.add(
      VideoFeedUpdate(
        revision: BigInt.one,
        phase: VideoFeedUpdatePhase.settled,
        hasPosts: true,
      ),
    );
    await pumpEventQueue();
    feed.release.complete();
    await refresh;
    await pumpEventQueue();

    final loaded = cubit.state as FeedLoaded;
    expect(feed.refreshCalls, 2);
    expect(loaded.posts.map((post) => post.id.value), ['initial', 'fresh']);
  });
}

final class _GatedRefreshFeed
    implements VideoFeedRepository, VideoFeedRefreshRepository {
  final initial = samplePost(id: 'initial');
  final fresh = samplePost(id: 'fresh');
  final firstStarted = Completer<void>();
  final release = Completer<void>();
  var refreshCalls = 0;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async => [initial];

  @override
  Future<VideoFeedRefreshSnapshot> loadRefresh(FeedKind kind) async {
    refreshCalls += 1;
    if (refreshCalls == 1) {
      firstStarted.complete();
      await release.future;
    }
    final posts = refreshCalls == 1 ? [initial] : [initial, fresh];
    return VideoFeedRefreshSnapshot(allPosts: posts, eligiblePosts: posts);
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async => VideoFeedPage(posts: const []);
}
