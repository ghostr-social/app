import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/account_scoped_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/repost_hydrated_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/controllable_video_feed_updates.dart';
import '../support/fakes.dart';
import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'a live revision keeps current rows and admits only fresh new rows',
    () async {
      final watched = samplePost(id: 'watched');
      final first = samplePost(id: 'first');
      final second = samplePost(id: 'second');
      final fresh = samplePost(id: 'fresh');
      final source = FakeVideoCatalogRepository(
        forYouFeed: [watched, first, second],
      );
      final history = FakeWatchHistoryRepository(
        entries: [
          WatchHistoryEntry.fromPost(watched, DateTime.utc(2026, 8, 1)),
        ],
      );
      final aware = WatchAwareVideoFeedRepository(
        feed: source,
        history: history,
        failureReporter: RecordingFailureReporter(),
      );
      final feed = AccountScopedVideoFeedRepository(
        RepostHydratedVideoFeedRepository(aware, source),
        () => NostrPublicKeyHex.parse(testViewerPublicKey),
      );
      final updates = ControllableVideoFeedUpdates();
      final cubit = FeedCubit(
        FeedDependencies(
          feed: feed,
          engagement: source,
          optional: FeedOptionalDependencies(
            watch: FeedWatchDependencies(
              tracker: WatchHistoryTracker(
                history: history,
                failureReporter: RecordingFailureReporter(),
              ),
            ),
            delivery: FeedDeliveryDependencies(updates: updates),
          ),
        ),
      );
      addTearDown(updates.close);
      addTearDown(cubit.close);
      await cubit.load();
      cubit.pageChanged(1);
      await pumpEventQueue();
      source.forYouFeed.insert(0, fresh);

      updates.add(
        VideoFeedUpdate(
          revision: BigInt.one,
          phase: VideoFeedUpdatePhase.loading,
          hasPosts: true,
        ),
      );
      await pumpEventQueue();

      final loaded = cubit.state as FeedLoaded;
      expect(loaded.posts.map((post) => post.id.value), [
        'first',
        'second',
        'fresh',
      ]);
      expect(loaded.roster.active.id.value, 'second');
    },
  );
}
