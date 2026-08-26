import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

import '../support/fake_feed_focus_port.dart';
import '../support/fakes.dart';
import '../support/feed_preparation_updates.dart';
import '../support/sample_data.dart';

void main() {
  test('exact WARP Ready prevents a stale delivery rescue', () async {
    final delivery = _DeliveryUpdates();
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
          delivery: FeedDeliveryDependencies(
            deliveryUpdates: delivery,
            preparationUpdates: preparation,
          ),
        ),
      ),
    );
    addTearDown(() async {
      await Future.wait([cubit.close(), delivery.close(), preparation.close()]);
    });
    await cubit.load();
    delivery.publish(posts[1], startable: false);
    delivery.publish(posts[2], startable: true);
    preparation.publish(_readyPlan(posts));

    cubit.pageChanged(1);
    await pumpEventQueue();
    delivery.publish(posts[2], startable: true);
    await pumpEventQueue();

    expect((cubit.state as FeedLoaded).roster.active.id.value, 'p1');
    expect(focus.focuses.last.cause, FeedFocusCause.userNavigation);
  });
}

PlaybackPreparationPlan _readyPlan(List<VideoPost> posts) {
  return PlaybackPreparationPlan(
    revision: BigInt.one,
    currentDeliveryId: posts.first.media.playbackDeliveryId,
    upcoming: [_readyAsset(posts[1].media)],
  );
}

PlaybackPreparationAsset _readyAsset(VideoMediaSource media) {
  const capability = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
  final deliveryId = media.playbackDeliveryId!;
  return PlaybackPreparationAsset(
    authority: PlaybackAssetAuthority(
      deliveryId: deliveryId,
      representationId: VideoRepresentationId.forMedia(media),
      assetId: PlaybackAssetId.parse(capability),
    ),
    media: ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:4040/video.mp4?'
      'id=${deliveryId.value}&cap=$capability',
    ),
    readiness: PlaybackPreparationReadiness.ready,
  );
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
