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
  test('a revision burst coalesces behind one passive reload', () async {
    final updates = ControllableVideoFeedUpdates();
    addTearDown(updates.close);
    final feed = _BlockedPassiveFeed();
    final cubit = FeedCubit(
      FeedDependencies(
        feed: feed,
        engagement: FakeVideoCatalogRepository(forYouFeed: []),
        optional: FeedOptionalDependencies(updates: updates),
      ),
    );
    addTearDown(cubit.close);
    await cubit.load();

    updates.add(_update(1));
    await pumpEventQueue();
    expect(feed.loadCalls, 2);
    updates
      ..add(_update(2))
      ..add(_update(3));
    await pumpEventQueue();

    expect(feed.loadCalls, 2);
    feed.passive.complete([samplePost(id: 'fresh')]);
    await pumpEventQueue();
    expect(feed.loadCalls, 3);
  });
}

VideoFeedUpdate _update(int revision) {
  return VideoFeedUpdate(
    revision: BigInt.from(revision),
    phase: VideoFeedUpdatePhase.loading,
    hasPosts: true,
  );
}

final class _BlockedPassiveFeed implements VideoFeedRepository {
  final passive = Completer<List<VideoPost>>();
  int loadCalls = 0;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) {
    loadCalls += 1;
    if (loadCalls == 1) return Future.value([samplePost(id: 'initial')]);
    if (loadCalls == 2) return passive.future;
    return Future.value([samplePost(id: 'latest')]);
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
