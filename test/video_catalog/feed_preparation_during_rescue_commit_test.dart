import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/controlled_video_delivery_updates.dart';
import '../support/fake_feed_focus_port.dart';
import '../support/fakes.dart';
import '../support/feed_preparation_updates.dart';
import '../support/ready_playback_preparation.dart';
import '../support/sample_data.dart';

void main() {
  test('watch persistence cannot delay or relabel a rescue', () async {
    final history = _GatedHistory();
    final delivery = ControlledVideoDeliveryUpdates();
    final preparation = ControlledPlaybackPreparationUpdates();
    final focus = FakeFeedFocusPort();
    final posts = List.generate(3, (index) => samplePost(id: 'p$index'));
    final source = FakeVideoCatalogRepository(forYouFeed: posts);
    final cubit = FeedCubit(
      FeedDependencies(
        feed: source,
        engagement: source,
        optional: FeedOptionalDependencies(
          focus: focus,
          watch: FeedWatchDependencies(
            tracker: WatchHistoryTracker(
              history: history,
              failureReporter: RecordingFailureReporter(),
            ),
          ),
          delivery: FeedDeliveryDependencies(
            deliveryUpdates: delivery,
            preparationUpdates: preparation,
          ),
        ),
      ),
    );
    addTearDown(() async {
      if (!history.release.isCompleted) history.release.complete();
      await Future.wait([cubit.close(), delivery.close(), preparation.close()]);
    });
    await cubit.load();
    delivery.publish(posts[1], phase: VideoDeliveryPhase.preparing);
    delivery.publish(posts[2], phase: VideoDeliveryPhase.startable);

    cubit.pageChanged(1);
    await history.secondStarted.future;
    await pumpEventQueue();

    expect((cubit.state as FeedLoaded).roster.active.id.value, 'p2');
    expect(focus.focuses.last.cause, FeedFocusCause.transportRescue);
    preparation.publish(_plan(posts[0].media, posts[1].media));
    await pumpEventQueue();
    expect((cubit.state as FeedLoaded).roster.active.id.value, 'p2');

    history.release.complete();
    await pumpEventQueue();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.preparation.isManaged, isTrue);
    expect(history.entries.map((entry) => entry.videoId), ['e:p2', 'e:p0']);
  });
}

PlaybackPreparationPlan _plan(
  VideoMediaSource current,
  VideoMediaSource intended,
) => PlaybackPreparationPlan(
  revision: BigInt.one,
  currentDeliveryId: current.playbackDeliveryId,
  upcoming: [readyPlaybackPreparation(intended)],
);

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
