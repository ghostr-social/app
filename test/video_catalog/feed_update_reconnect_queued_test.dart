import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_update_retry.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/scripted_video_feed_updates.dart';

void main() {
  test('same-kind reconnect preserves a queued reconciliation', () async {
    final first = StreamController<VideoFeedUpdate>();
    final second = StreamController<VideoFeedUpdate>();
    addTearDown(second.close);
    final updates = ScriptedVideoFeedUpdates([
      () => first.stream,
      () => second.stream,
    ]);
    final feed = _QueuedReconnectFeed();
    final cubit = FeedCubit(
      FeedDependencies(
        feed: feed,
        engagement: FakeVideoCatalogRepository(forYouFeed: []),
        optional: FeedOptionalDependencies(updates: updates),
      ),
      updateRetry: FeedUpdateRetry(delays: const [Duration.zero]),
    );
    addTearDown(cubit.close);

    final load = cubit.load();
    await pumpEventQueue();
    first.add(
      VideoFeedUpdate(
        revision: BigInt.one,
        phase: VideoFeedUpdatePhase.loading,
        hasPosts: true,
      ),
    );
    await pumpEventQueue();
    await first.close();
    await pumpEventQueue();
    expect(updates.watchCalls, 2);

    feed.initial.complete(const []);
    feed.reconciliation.complete([samplePost(id: 'reconciled')]);
    await load;
    await pumpEventQueue();

    expect(cubit.state, isA<FeedLoaded>());
    expect(feed.loadCalls, 2);
  });
}

final class _QueuedReconnectFeed implements VideoFeedRepository {
  final initial = Completer<List<VideoPost>>();
  final reconciliation = Completer<List<VideoPost>>();
  int loadCalls = 0;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) {
    loadCalls += 1;
    return loadCalls == 1 ? initial.future : reconciliation.future;
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async {
    return VideoFeedPage(posts: const []);
  }
}
