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
  test(
    'a swipe keeps the older page already loading for its new tail',
    () async {
      final feed = _GatedBackfillFeed();
      final history = _GatedHistory();
      final cubit = FeedCubit(
        FeedDependencies(
          feed: feed,
          engagement: FakeVideoCatalogRepository(forYouFeed: const []),
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
        await cubit.close();
      });
      await cubit.load();
      await feed.olderStarted.future;

      cubit.pageChanged(1);
      await history.secondStarted.future;
      feed.older.complete(VideoFeedPage(posts: [samplePost(id: 'p2')]));
      await pumpEventQueue();
      history.release.complete();
      await pumpEventQueue();

      final loaded = cubit.state as FeedLoaded;
      expect(loaded.posts.map((post) => post.id.value), ['p1', 'p2']);
    },
  );
}

final class _GatedHistory extends FakeWatchHistoryRepository {
  final secondStarted = Completer<void>();
  final release = Completer<void>();
  var writes = 0;

  @override
  Future<void> record(WatchHistoryEntry entry) async {
    if (++writes == 2) {
      secondStarted.complete();
      await release.future;
    }
    await super.record(entry);
  }
}

final class _GatedBackfillFeed implements VideoFeedRepository {
  final olderStarted = Completer<void>();
  final older = Completer<VideoFeedPage>();

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    return [samplePost(id: 'p0'), samplePost(id: 'p1')];
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) {
    olderStarted.complete();
    return older.future;
  }
}
