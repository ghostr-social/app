import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/platform/media/gateway_playback_cubit.dart';

import '../support/fake_progressive_playback_gateway.dart';

void main() {
  test('prepared authority supersedes a pending generic resolution', () async {
    final gateway = FakeProgressivePlaybackGateway();
    final origin = VideoMediaSource.withCacheScope(
      VideoMediaSource.remote('https://media.test/clip.mp4'),
      'p0',
    );
    final prepared = _asset(origin).bind(origin);
    final cubit = GatewayPlaybackCubit(gateway);
    final pending = cubit.load(origin);

    await cubit.load(origin, prepared: prepared);
    gateway.completeNext(playbackUrl: _lateUrl);
    await pending;

    final ready = cubit.state as GatewayPlaybackReady;
    expect(ready.media, same(prepared.media));
    expect(ready.origin, same(origin));
    await cubit.close();
  });
}

PlaybackPreparationAsset _asset(VideoMediaSource origin) {
  const cap = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
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

const _lateUrl =
    'http://127.0.0.1:4040/video.mp4?id=p0&cap='
    'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
