import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/feed_preparation_updates.dart';
import '../support/sample_data.dart';

void main() {
  test('preparation update cannot cancel a durable rescue commit', () async {
    final history = _GatedHistory();
    final delivery = _DeliveryUpdates();
    final preparation = ControlledPlaybackPreparationUpdates();
    final posts = List.generate(3, (index) => samplePost(id: 'p$index'));
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
    delivery.publish(posts[1], startable: false);
    delivery.publish(posts[2], startable: true);

    cubit.pageChanged(1);
    await history.secondStarted.future;
    preparation.publish(_plan('p0'));
    history.release.complete();
    await pumpEventQueue();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts.first.id.value, 'p2');
    expect(loaded.preparation.isManaged, isTrue);
  });
}

PlaybackPreparationPlan _plan(String current) => PlaybackPreparationPlan(
  revision: BigInt.one,
  currentDeliveryId: PlaybackDeliveryId.parse(current),
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

final class _DeliveryUpdates implements VideoDeliveryUpdates {
  final _events = StreamController<VideoDeliverySnapshot>.broadcast(sync: true);

  @override
  Stream<VideoDeliverySnapshot> watchDelivery() => _events.stream;

  void publish(VideoPost post, {required bool startable}) {
    _events.add(
      VideoDeliverySnapshot(
        deliveryId: post.media.playbackDeliveryId!,
        phase: startable
            ? VideoDeliveryPhase.startable
            : VideoDeliveryPhase.preparing,
        bytesPresent: BigInt.zero,
      ),
    );
  }

  Future<void> close() => _events.close();
}
