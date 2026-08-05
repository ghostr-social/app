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
  test('startup replay does not interrupt the initial backfill', () async {
    final updates = ControllableVideoFeedUpdates();
    addTearDown(updates.close);
    final feed = _StartupFeed();
    final cubit = FeedCubit(
      FeedDependencies(
        feed: feed,
        engagement: FakeVideoCatalogRepository(forYouFeed: []),
        optional: FeedOptionalDependencies(updates: updates),
      ),
    );
    addTearDown(cubit.close);

    final load = cubit.load();
    await pumpEventQueue();
    updates.add(
      VideoFeedUpdate(
        revision: BigInt.one,
        phase: VideoFeedUpdatePhase.settled,
        hasPosts: true,
      ),
    );
    await pumpEventQueue();
    feed.initial.complete([samplePost(id: 'initial')]);
    await load;
    await pumpEventQueue();
    feed.older.complete(VideoFeedPage(posts: [samplePost(id: 'older')]));
    await pumpEventQueue();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts.map((post) => post.id.value), ['initial', 'older']);
    expect(feed.loadCalls, 1);
  });
}

final class _StartupFeed implements VideoFeedRepository {
  final initial = Completer<List<VideoPost>>();
  final older = Completer<VideoFeedPage>();
  int loadCalls = 0;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) {
    loadCalls += 1;
    if (loadCalls == 1) return initial.future;
    return Future.value([samplePost(id: 'initial')]);
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) {
    return older.future;
  }
}
