import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fake_feed_focus_port.dart';
import '../support/fakes.dart';
import '../support/feed_preparation_updates.dart';
import '../support/player_verified_preparation.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'default decoder failure preserves the intended semantic item',
    () async {
      final updates = _DeliveryUpdates();
      final preparation = ControlledPlaybackPreparationUpdates();
      final focus = FakeFeedFocusPort();
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
          ),
        ),
      );
      addTearDown(cubit.close);
      addTearDown(updates.close);
      addTearDown(preparation.close);
      await cubit.load();
      updates.publish(posts[0], VideoDeliveryPhase.startable);
      updates.publish(posts[1], VideoDeliveryPhase.startable);
      preparation.publish(
        playerVerifiedPlan(posts, currentIndex: 0, readyIndices: [1]),
      );

      updates.publish(posts[0], VideoDeliveryPhase.failed);
      await pumpEventQueue();

      expect((cubit.state as FeedLoaded).activeIndex, 0);
      expect(
        focus.focuses.any(
          (event) => event.cause == FeedFocusCause.transportRescue,
        ),
        isFalse,
      );
    },
  );
}

final class _DeliveryUpdates implements VideoDeliveryUpdates {
  final _events = StreamController<VideoDeliverySnapshot>.broadcast(sync: true);

  @override
  Stream<VideoDeliverySnapshot> watchDelivery() => _events.stream;

  void publish(VideoPost post, VideoDeliveryPhase phase) {
    _events.add(
      VideoDeliverySnapshot(
        deliveryId: post.media.playbackDeliveryId!,
        phase: phase,
        bytesPresent: BigInt.zero,
      ),
    );
  }

  Future<void> close() => _events.close();
}
