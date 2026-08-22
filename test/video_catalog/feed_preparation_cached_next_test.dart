import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_preparation_reducer.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

void main() {
  test('a cached next item is never bound as prepared progressive media', () {
    final cached = VideoMediaSource.cached(
      '/cache/next.mp4',
      remoteUrl: _remoteUrl,
    );
    final asset = _asset(VideoMediaSource.remote(_remoteUrl));
    final plan = PlaybackPreparationPlan(
      revision: BigInt.one,
      currentDeliveryId: asset.deliveryId,
      current: asset,
    );
    final hls = VideoMediaSource.remote(
      'https://media.test/live.m3u8',
      delivery: VideoMediaDelivery.hls,
    );

    final result = FeedPreparationReducer().accept(plan, hls, cached);

    expect(result?.isManaged, isFalse);
    expect(result?.next, isNull);
  });
}

PlaybackPreparationAsset _asset(VideoMediaSource origin) {
  const cap = 'ddddddddddddddddddddddddddddddddddddddddddd';
  return PlaybackPreparationAsset(
    authority: PlaybackAssetAuthority(
      deliveryId: origin.playbackDeliveryId!,
      representationId: VideoRepresentationId.forMedia(origin),
      assetId: PlaybackAssetId.parse(cap),
    ),
    media: ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:4040/video.mp4?'
      'id=${origin.playbackDeliveryId!.value}&cap=$cap',
    ),
    readiness: PlaybackPreparationReadiness.structuralStartable,
  );
}

const _remoteUrl = 'https://media.test/next.mp4';
