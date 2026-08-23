import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/platform/media/gateway_playback_cubit.dart';

import '../support/fake_progressive_playback_gateway.dart';

void main() {
  test('renewed prepared capability replaces the active exact asset', () async {
    final origin = VideoMediaSource.withCacheScope(
      VideoMediaSource.remote('https://media.test/clip.mp4'),
      'p0',
    );
    final cubit = GatewayPlaybackCubit(FakeProgressivePlaybackGateway());
    final first = _asset(origin, _firstCapability).bind(origin);
    final renewed = _asset(origin, _renewedCapability).bind(origin);

    await cubit.load(origin, prepared: first);
    await cubit.load(origin, prepared: renewed);

    final ready = cubit.state as GatewayPlaybackReady;
    expect(ready.media, same(renewed.media));
    await cubit.close();
  });
}

PlaybackPreparationAsset _asset(VideoMediaSource origin, String capability) {
  return PlaybackPreparationAsset(
    authority: PlaybackAssetAuthority(
      deliveryId: PlaybackDeliveryId.parse('p0'),
      representationId: VideoRepresentationId.forMedia(origin),
      assetId: PlaybackAssetId.parse(capability),
    ),
    media: ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:4040/video.mp4?id=p0&cap=$capability',
    ),
    readiness: PlaybackPreparationReadiness.preparing,
  );
}

const _firstCapability = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const _renewedCapability = 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
