import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_hunt.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/controllable_video_feed_updates.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a watch failure stops live update reconciliation', () async {
    final updates = ControllableVideoFeedUpdates();
    final source = _RetryDetectingFeed(samplePost(id: 'fresh'));
    final cubit = FeedCubit(
      FeedDependencies(
        feed: source,
        engagement: FakeVideoCatalogRepository(forYouFeed: const []),
        optional: FeedOptionalDependencies(
          delivery: FeedDeliveryDependencies(updates: updates),
          watch: FeedWatchDependencies(
            tracker: WatchHistoryTracker(
              history: _FailingHistory(),
              failureReporter: RecordingFailureReporter(),
            ),
          ),
        ),
      ),
      hunt: FeedHunt(base: const Duration(days: 1)),
    );
    addTearDown(() async {
      if (!source.release.isCompleted) source.release.complete();
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
    await pumpEventQueue();

    expect(cubit.state, isA<FeedFailure>());
    expect(source.unexpectedRetry.isCompleted, isFalse);
  });
}

final class _RetryDetectingFeed implements VideoFeedRepository {
  _RetryDetectingFeed(this.fresh);

  final VideoPost fresh;
  final unexpectedRetry = Completer<void>();
  final release = Completer<void>();
  var calls = 0;

  @override
  Future<List<VideoPost>> loadFeed(
    FeedKind kind, {
    bool excludeWatched = false,
  }) async {
    calls += 1;
    if (calls == 1) return [];
    if (calls == 2) return [fresh];
    unexpectedRetry.complete();
    await release.future;
    return [fresh];
  }

  @override
  Future<VideoFeedPage> loadOlderFeed(
    FeedKind kind, {
    required DateTime olderThan,
    bool excludeWatched = false,
  }) async => VideoFeedPage(posts: const []);
}

final class _FailingHistory extends FakeWatchHistoryRepository {
  @override
  Future<void> record(WatchHistoryEntry entry) async {
    throw const AppFailure('Storage unavailable.');
  }
}
