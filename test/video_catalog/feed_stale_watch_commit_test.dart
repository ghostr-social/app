import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a stale watch commit cannot overtake a newer load', () async {
    final post = samplePost();
    final feed = _SecondLoadGatedFeed(post);
    final history = _FirstWatchGatedHistory();
    final cubit = FeedCubit(
      FeedDependencies(
        feed: feed,
        engagement: FakeVideoCatalogRepository(forYouFeed: []),
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
      if (!feed.release.isCompleted) feed.release.complete();
      await cubit.close();
    });

    final stale = cubit.load();
    await history.started.future;
    final newest = cubit.load();
    await feed.started.future;
    history.release.complete();
    await stale;

    expect(cubit.state, isA<FeedLoading>());
    feed.release.complete();
    await newest;
    expect(cubit.state, isA<FeedLoaded>());
  });
}

final class _FirstWatchGatedHistory extends FakeWatchHistoryRepository {
  final started = Completer<void>();
  final release = Completer<void>();

  @override
  Future<void> record(WatchHistoryEntry entry) async {
    if (!started.isCompleted) {
      started.complete();
      await release.future;
    }
    await super.record(entry);
  }
}

final class _SecondLoadGatedFeed implements VideoFeedRepository {
  _SecondLoadGatedFeed(this.post);

  final VideoPost post;
  final started = Completer<void>();
  final release = Completer<void>();
  var loads = 0;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    loads += 1;
    if (loads == 2) {
      started.complete();
      await release.future;
    }
    return [post];
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async => VideoFeedPage(posts: const []);
}
