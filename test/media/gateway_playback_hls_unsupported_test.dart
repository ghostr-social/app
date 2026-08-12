import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/gateway_video_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_progressive_playback_gateway.dart';

void main() {
  testWidgets('labels remote HLS unsupported without a futile retry', (
    tester,
  ) async {
    final gateway = FakeProgressivePlaybackGateway();
    final playback = GatewayVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      gateway: gateway,
    );
    final media = VideoMediaSource.remote(
      'https://media.test/live.m3u8',
      delivery: VideoMediaDelivery.hls,
    );

    await tester.pumpWidget(
      MaterialApp(
        home: playback.buildSurface(
          VideoPlaybackSurfaceRequest(media: media, isActive: true),
        ),
      ),
    );

    expect(find.text('Streaming video unsupported'), findsOneWidget);
    expect(find.text('Retry'), findsNothing);
    expect(gateway.requests, isEmpty);
  });
}
