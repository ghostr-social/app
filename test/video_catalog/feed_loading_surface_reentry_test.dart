import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'an offscreen loading feed is abandoned and reloaded on return',
    () async {
      final post = samplePost();
      final source = _GatedFeed(post);
      final history = FakeWatchHistoryRepository();
      final cubit = FeedCubit(
        FeedDependencies(
          feed: source,
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
      addTearDown(cubit.close);

      final initialLoad = cubit.load();
      await source.started.future;
      cubit.surfaceVisibilityChanged(false);
      source.release.complete();
      await initialLoad;
      expect(history.entries, isEmpty);

      cubit.surfaceVisibilityChanged(true);
      await pumpEventQueue();

      expect(cubit.state, isA<FeedLoaded>());
      expect(history.entries.single.videoId, 'e:post-1');
    },
  );
}

final class _GatedFeed implements VideoFeedRepository {
  _GatedFeed(this.post);

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
    if (loads == 1) {
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
