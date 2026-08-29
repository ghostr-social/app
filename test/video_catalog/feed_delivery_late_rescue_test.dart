import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fake_feed_focus_port.dart';
import '../support/fakes.dart';
import '../support/feed_preparation_updates.dart';
import '../support/player_verified_preparation.dart';
import '../support/sample_data.dart';

void main() {
  test('a ready neighbor rescues a stalled no-replay swipe', () async {
    final updates = _DeliveryUpdates();
    final preparation = ControlledPlaybackPreparationUpdates();
    final focus = FakeFeedFocusPort();
    final history = FakeWatchHistoryRepository();
    final posts = List.generate(3, (index) => samplePost(id: 'p$index'));
    final repository = FakeVideoCatalogRepository(forYouFeed: posts);
    final cubit = FeedCubit(
      FeedDependencies(
        feed: repository,
        engagement: repository,
        optional: FeedOptionalDependencies(
          focus: focus,
          delivery: FeedDeliveryDependencies(
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
    addTearDown(cubit.close);
    addTearDown(updates.close);
    addTearDown(preparation.close);
    await cubit.load();

    cubit.pageChanged(1);
    await pumpEventQueue();
    expect((cubit.state as FeedLoaded).roster.active.id.value, 'p1');
    updates.publish(posts[1], startable: false);
    updates.publish(posts[2], startable: true);
    preparation.publish(
      playerVerifiedPlan(posts, currentIndex: 1, readyIndices: [2]),
    );
    await pumpEventQueue();

    expect((cubit.state as FeedLoaded).roster.active.id.value, 'p2');
    expect(focus.focuses.last.cause, FeedFocusCause.transportRescue);
    expect(history.entries.map((entry) => entry.videoId), [
      'e:p2',
      'e:p1',
      'e:p0',
    ]);
  });
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
