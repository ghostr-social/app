import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

void main() {
  test('binds an exact asset only to its represented origin', () {
    final represented = _origin('https://media.test/a.mp4');
    final different = _origin('https://media.test/b.mp4');
    final asset = _asset(represented);

    final prepared = asset.bind(represented);

    expect(prepared.origin, same(represented));
    expect(prepared.media, same(asset.media));
    expect(() => asset.bind(different), throwsArgumentError);
    expect(
      () => PreparedProgressiveVideoPlaybackRequest(
        request: VideoPlaybackSurfaceRequest(
          media: different,
          isActive: true,
        ),
        prepared: prepared,
      ),
      throwsArgumentError,
    );
  });
}

VideoMediaSource _origin(String url) {
  return VideoMediaSource.withCacheScope(VideoMediaSource.remote(url), 'p0');
}

PlaybackPreparationAsset _asset(VideoMediaSource origin) {
  const capability = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
  return PlaybackPreparationAsset(
    authority: PlaybackAssetAuthority(
      deliveryId: PlaybackDeliveryId.parse('p0'),
      representationId: VideoRepresentationId.forMedia(origin),
      assetId: PlaybackAssetId.parse(capability),
    ),
    media: ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:4040/video.mp4?id=p0&cap=$capability',
    ),
    readiness: PlaybackPreparationReadiness.structuralStartable,
  );
}
