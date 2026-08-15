import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fake_feed_focus_port.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a newly ready neighbor rescues an already stalled swipe', () async {
    final updates = _DeliveryUpdates();
    final focus = FakeFeedFocusPort();
    final posts = List.generate(3, (index) => samplePost(id: 'p$index'));
    final repository = FakeVideoCatalogRepository(forYouFeed: posts);
    final cubit = FeedCubit(
      FeedDependencies(
        feed: repository,
        engagement: repository,
        optional: FeedOptionalDependencies(
          focus: focus,
          delivery: FeedDeliveryDependencies(deliveryUpdates: updates),
        ),
      ),
    );
    addTearDown(cubit.close);
    addTearDown(updates.close);
    await cubit.load();

    cubit.pageChanged(1);
    expect((cubit.state as FeedLoaded).activeIndex, 1);
    updates.publish(posts[1], startable: false);
    updates.publish(posts[2], startable: true);
    await pumpEventQueue();

    expect((cubit.state as FeedLoaded).activeIndex, 2);
    expect(focus.focuses.last.cause, FeedFocusCause.transportRescue);
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
