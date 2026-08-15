import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a short ETA waits once, then rescues to ready media', () async {
    final updates = _DeliveryUpdates();
    final posts = List.generate(3, (index) => samplePost(id: 'p$index'));
    final repository = FakeVideoCatalogRepository(forYouFeed: posts);
    final cubit = FeedCubit(
      FeedDependencies(
        feed: repository,
        engagement: repository,
        optional: FeedOptionalDependencies(
          delivery: FeedDeliveryDependencies(deliveryUpdates: updates),
        ),
      ),
    );
    addTearDown(cubit.close);
    addTearDown(updates.close);
    await cubit.load();

    fakeAsync((clock) {
      updates.publish(posts[1], ready: false, etaMs: 100);
      updates.publish(posts[2], ready: true, etaMs: 0);
      cubit.pageChanged(1);
      expect((cubit.state as FeedLoaded).activeIndex, 1);

      clock.elapse(const Duration(milliseconds: 250));

      expect((cubit.state as FeedLoaded).activeIndex, 2);
    });
  });
}

final class _DeliveryUpdates implements VideoDeliveryUpdates {
  final _events = StreamController<VideoDeliverySnapshot>.broadcast(sync: true);

  @override
  Stream<VideoDeliverySnapshot> watchDelivery() => _events.stream;

  void publish(VideoPost post, {required bool ready, required int etaMs}) {
    _events.add(
      VideoDeliverySnapshot(
        deliveryId: post.media.playbackDeliveryId!,
        phase: ready
            ? VideoDeliveryPhase.startable
            : VideoDeliveryPhase.preparing,
        bytesPresent: BigInt.zero,
        eta: Duration(milliseconds: etaMs),
      ),
    );
  }

  Future<void> close() => _events.close();
}
