import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/feed_preparation_updates.dart';
import '../support/sample_data.dart';

void main() {
  test('preparation update cannot cancel a durable page commit', () async {
    final history = _GatedHistory();
    final updates = ControlledPlaybackPreparationUpdates();
    final posts = [samplePost(id: 'first'), samplePost(id: 'second')];
    final source = FakeVideoCatalogRepository(forYouFeed: posts);
    final cubit = FeedCubit(
      FeedDependencies(
        feed: source,
        engagement: source,
        optional: FeedOptionalDependencies(
          watch: FeedWatchDependencies(
            tracker: WatchHistoryTracker(
              history: history,
              failureReporter: RecordingFailureReporter(),
            ),
          ),
          delivery: FeedDeliveryDependencies(preparationUpdates: updates),
        ),
      ),
    );
    addTearDown(() async {
      if (!history.release.isCompleted) history.release.complete();
      await cubit.close();
      await updates.close();
    });
    await cubit.load();

    cubit.pageChanged(1);
    await history.secondStarted.future;
    updates.publish(
      PlaybackPreparationPlan(
        revision: BigInt.one,
        currentDeliveryId: PlaybackDeliveryId.parse('first'),
      ),
    );
    history.release.complete();
    await pumpEventQueue();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.roster.active.id.value, 'second');
    expect(loaded.preparation.isManaged, isTrue);
  });
}

final class _GatedHistory extends FakeWatchHistoryRepository {
  final secondStarted = Completer<void>();
  final release = Completer<void>();
  var writes = 0;

  @override
  Future<void> record(WatchHistoryEntry entry) async {
    writes += 1;
    if (writes == 2) {
      secondStarted.complete();
      await release.future;
    }
    await super.record(entry);
  }
}
