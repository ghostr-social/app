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
  test('a revision landing during the first pull is reconciled', () async {
    final updates = ControllableVideoFeedUpdates();
    addTearDown(updates.close);
    final feed = _BlockedFirstFeed();
    final cubit = FeedCubit(
      FeedDependencies(
        feed: feed,
        engagement: FakeVideoCatalogRepository(forYouFeed: []),
        optional: FeedOptionalDependencies(
          delivery: FeedDeliveryDependencies(updates: updates),
        ),
      ),
    );
    addTearDown(cubit.close);

    final load = cubit.load();
    await pumpEventQueue();
    updates.add(
      VideoFeedUpdate(
        revision: BigInt.one,
        phase: VideoFeedUpdatePhase.loading,
        hasPosts: true,
      ),
    );
    await pumpEventQueue();
    feed.first.complete(const []);
    await load;
    await pumpEventQueue();

    expect(cubit.state, isA<FeedLoaded>());
    expect(feed.loadCalls, 2);
  });
}

final class _BlockedFirstFeed implements VideoFeedRepository {
  final first = Completer<List<VideoPost>>();
  int loadCalls = 0;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) {
    loadCalls += 1;
    if (loadCalls == 1) return first.future;
    return Future.value([samplePost(id: 'late')]);
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
