import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/controlled_video_delivery_updates.dart';
import '../support/fakes.dart';
import '../support/feed_preparation_updates.dart';
import '../support/player_verified_preparation.dart';
import '../support/sample_data.dart';

void main() {
  test('grace rescue starts after a no-replay watch commits', () {
    fakeAsync((clock) {
      final updates = ControlledVideoDeliveryUpdates();
      final preparation = ControlledPlaybackPreparationUpdates();
      final history = _GatedHistory();
      final posts = List.generate(3, (index) => samplePost(id: 'p$index'));
      final repository = FakeVideoCatalogRepository(forYouFeed: posts);
      final cubit = FeedCubit(
        FeedDependencies(
          feed: repository,
          engagement: repository,
          optional: FeedOptionalDependencies(
            delivery: FeedDeliveryDependencies.withReadyRescue(
              deliveryUpdates: updates,
              preparationUpdates: preparation,
            ),
            watch: FeedWatchDependencies(
              tracker: WatchHistoryTracker(
                history: history,
                failureReporter: RecordingFailureReporter(),
              ),
            ),
          ),
        ),
      );
      cubit.load();
      clock.flushMicrotasks();
      updates.publish(posts[0], phase: VideoDeliveryPhase.startable);
      updates.publish(
        posts[1],
        phase: VideoDeliveryPhase.preparing,
        eta: const Duration(milliseconds: 100),
      );
      preparation.publish(
        playerVerifiedPlan(posts, currentIndex: 0, readyIndices: [2]),
      );
      cubit.pageChanged(1);
      clock.flushMicrotasks();
      expect((cubit.state as FeedLoaded).roster.active.id.value, 'p0');

      clock.elapse(const Duration(milliseconds: 250));
      history.release.complete();
      clock.flushMicrotasks();
      expect((cubit.state as FeedLoaded).roster.active.id.value, 'p1');
      clock.elapse(const Duration(milliseconds: 250));

      expect((cubit.state as FeedLoaded).roster.active.id.value, 'p2');
      cubit.close();
      updates.close();
      preparation.close();
      clock.flushMicrotasks();
    });
  });
}

final class _GatedHistory extends FakeWatchHistoryRepository {
  final release = Completer<void>();
  var writes = 0;

  @override
  Future<void> record(WatchHistoryEntry entry) async {
    writes += 1;
    if (writes == 2) await release.future;
    await super.record(entry);
  }
}
