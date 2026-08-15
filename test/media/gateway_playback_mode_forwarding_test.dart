import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/gateway_video_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_progressive_playback_gateway.dart';

void main() {
  testWidgets('progressive gateway forwards interactive playback mode', (
    tester,
  ) async {
    final delegate = FakeVideoPlaybackPort();
    final gateway = FakeProgressivePlaybackGateway();
    final playback = GatewayVideoPlaybackPort(
      delegate: delegate,
      gateway: gateway,
    );

    await tester.pumpWidget(
      MaterialApp(
        home: playback.buildSurface(
          VideoPlaybackSurfaceRequest(
            media: VideoMediaSource.remote('https://media.test/clip.mp4'),
            isActive: true,
            mode: VideoPlaybackMode.accelerated,
          ),
        ),
      ),
    );
    gateway.completeNext();
    await tester.pump();

    expect(delegate.requests.last.mode, VideoPlaybackMode.accelerated);
  });
}
