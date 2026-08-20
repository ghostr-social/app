import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/core/media/video_media_metadata.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/platform/media/gateway_playback_cubit.dart';

import '../support/fake_progressive_playback_gateway.dart';

void main() {
  test('metadata revision replaces a ready exact representation', () async {
    final previous = _origin(10);
    final current = _origin(11);
    final gateway = FakeProgressivePlaybackGateway(
      immediatePlaybackUrl: fakeProgressivePlaybackUrl,
    );
    final cubit = GatewayPlaybackCubit(gateway);
    await cubit.load(previous);

    expect(
      previous.inventoryPlaybackIdentity,
      current.inventoryPlaybackIdentity,
    );
    expect(
      VideoRepresentationId.forMedia(previous),
      isNot(VideoRepresentationId.forMedia(current)),
    );

    final prepared = _asset(current).bind(current);
    await cubit.load(current, prepared: prepared);

    final ready = cubit.state as GatewayPlaybackReady;
    expect(ready.origin, same(current));
    expect(ready.media, same(prepared.media));
    await cubit.close();
  });
}

VideoMediaSource _origin(int sizeBytes) {
  return VideoMediaSource.withCacheScope(
    VideoMediaSource.remote(
      'https://media.test/clip.mp4',
      metadata: VideoMediaMetadata(sizeBytes: sizeBytes, durationMs: 1000),
    ),
    'p0',
  );
}

PlaybackPreparationAsset _asset(VideoMediaSource origin) {
  const cap = 'ccccccccccccccccccccccccccccccccccccccccccc';
  return PlaybackPreparationAsset(
    authority: PlaybackAssetAuthority(
      deliveryId: PlaybackDeliveryId.parse('p0'),
      representationId: VideoRepresentationId.forMedia(origin),
      assetId: PlaybackAssetId.parse(cap),
    ),
    media: ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:4040/video.mp4?id=p0&cap=$cap',
    ),
    readiness: PlaybackPreparationReadiness.preparing,
  );
}
