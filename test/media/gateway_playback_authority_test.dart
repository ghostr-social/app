import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/prepared_progressive_playback.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/platform/media/gateway_video_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import '../support/fake_progressive_playback_gateway.dart';
import '../support/recording_video_playback_port.dart';

void main() {
  testWidgets('prepared gateway playback preserves exact asset authority', (
    tester,
  ) async {
    final delegate = RecordingVideoPlaybackPort();
    final prepared = _prepared();
    final port = GatewayVideoPlaybackPort(
      delegate: delegate,
      gateway: FakeProgressivePlaybackGateway(),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: port.buildSurface(
          PreparedProgressiveVideoPlaybackRequest(
            request: VideoPlaybackSurfaceRequest(
              media: prepared.origin,
              isActive: false,
            ),
            prepared: prepared,
          ),
        ),
      ),
    );
    await tester.pump();

    expect(delegate.requests.single.authority, prepared.authority);
  });
}

PreparedProgressivePlayback _prepared() {
  const cap = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
  final origin = VideoMediaSource.withCacheScope(
    VideoMediaSource.remote('https://media.test/clip.mp4'),
    'post-1',
  );
  final authority = PlaybackAssetAuthority(
    deliveryId: PlaybackDeliveryId.parse('post-1'),
    representationId: VideoRepresentationId.forMedia(origin),
    assetId: PlaybackAssetId.parse(cap),
  );
  return PreparedProgressivePlayback.bind(
    origin: origin,
    media: ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:4040/video.mp4?id=post-1&cap=$cap',
    ),
    authority: authority,
  );
}
