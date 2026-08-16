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
  test('a Home reload supersedes an in-flight raw resync', () async {
    final updates = ControllableVideoFeedUpdates();
    final feed = _GatedRefreshFeed();
    final cubit = FeedCubit(
      FeedDependencies(
        feed: feed,
        engagement: FakeVideoCatalogRepository(forYouFeed: []),
        optional: FeedOptionalDependencies(
          delivery: FeedDeliveryDependencies(updates: updates),
        ),
      ),
    );
    addTearDown(() async {
      if (!feed.reload.isCompleted) feed.reload.complete([feed.fresh]);
      await cubit.close();
      await updates.close();
    });
    await cubit.load();
    updates.add(
      VideoFeedUpdate(
        revision: BigInt.one,
        phase: VideoFeedUpdatePhase.settled,
        hasPosts: true,
      ),
    );
    await feed.refreshStarted.future;
    final admitted = <String>[];
    var loadingStarted = false;
    final subscription = cubit.stream.listen((state) {
      if (loadingStarted && state is FeedLoaded) {
        admitted.addAll(state.posts.map((post) => post.id.value));
      }
      if (state is FeedLoading && !feed.refresh.isCompleted) {
        loadingStarted = true;
        feed.refresh.complete(
          VideoFeedRefreshSnapshot(
            allPosts: [feed.watched, feed.fresh],
            eligiblePosts: [feed.fresh],
          ),
        );
      }
    });
    addTearDown(subscription.cancel);
    final loading = cubit.reload();
    await pumpEventQueue();

    expect(cubit.state, isA<FeedLoading>());
    expect(admitted, isNot(contains('watched')));
    feed.reload.complete([feed.fresh]);
    await loading;
  });
}

final class _GatedRefreshFeed
    implements VideoFeedRepository, VideoFeedRefreshRepository {
  final watched = samplePost(id: 'watched');
  final fresh = samplePost(id: 'fresh');
  final refreshStarted = Completer<void>();
  final refresh = Completer<VideoFeedRefreshSnapshot>();
  final reload = Completer<List<VideoPost>>();
  var loads = 0;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) {
    loads += 1;
    return loads == 1 ? Future.value([watched]) : reload.future;
  }

  @override
  Future<VideoFeedRefreshSnapshot> loadRefresh(FeedKind kind) {
    refreshStarted.complete();
    return refresh.future;
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async => VideoFeedPage(posts: const []);
}
