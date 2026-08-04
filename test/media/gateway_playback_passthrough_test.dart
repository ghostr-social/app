import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/gateway_video_playback_port.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_progressive_playback_gateway.dart';

void main() {
  testWidgets('passes local media through without a gateway lookup',
      (tester) async {
    final gateway = FakeProgressivePlaybackGateway();
    final playback = GatewayVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      gateway: gateway,
    );

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(
        media: VideoMediaSource.local('/cache/video.mp4'),
        isActive: true,
      ),
    ));

    expect(find.text('/cache/video.mp4'), findsOneWidget);
    expect(gateway.requests, isEmpty);
  });

  testWidgets('passes already-proxied media through untouched',
      (tester) async {
    final gateway = FakeProgressivePlaybackGateway();
    final playback = GatewayVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      gateway: gateway,
    );

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(
        media: ProxiedProgressiveVideoMediaSource(fakeProgressivePlaybackUrl),
        isActive: true,
      ),
    ));

    expect(find.text('Progressive loopback stream'), findsOneWidget);
    expect(gateway.requests, isEmpty);
  });
}
