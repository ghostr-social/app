import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/controllable_video_feed_updates.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'a hidden in-flight live update retries when the feed returns',
    () async {
      final updates = ControllableVideoFeedUpdates();
      final feed = _GatedUpdateFeed();
      final cubit = FeedCubit(
        FeedDependencies(
          feed: feed,
          engagement: FakeVideoCatalogRepository(forYouFeed: const []),
          optional: FeedOptionalDependencies(
            delivery: FeedDeliveryDependencies(updates: updates),
          ),
        ),
      );
      addTearDown(cubit.close);
      addTearDown(updates.close);
      await cubit.load();

      updates.add(
        VideoFeedUpdate(
          revision: BigInt.one,
          phase: VideoFeedUpdatePhase.settled,
          hasPosts: true,
        ),
      );
      await feed.refreshStarted.future;
      cubit.surfaceVisibilityChanged(false);
      feed.releaseRefresh.complete();
      await pumpEventQueue();
      cubit.surfaceVisibilityChanged(true);
      await pumpEventQueue();

      final loaded = cubit.state as FeedLoaded;
      expect(feed.loadCalls, 3);
      expect(loaded.posts.map((post) => post.id.value), ['initial', 'fresh']);
    },
  );
}

final class _GatedUpdateFeed implements VideoFeedRepository {
  final initial = samplePost(id: 'initial');
  final fresh = samplePost(id: 'fresh');
  final refreshStarted = Completer<void>();
  final releaseRefresh = Completer<void>();
  var loadCalls = 0;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    loadCalls += 1;
    if (loadCalls == 1) return [initial];
    if (loadCalls == 2) {
      refreshStarted.complete();
      await releaseRefresh.future;
    }
    return [initial, fresh];
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async => VideoFeedPage(posts: const []);
}
