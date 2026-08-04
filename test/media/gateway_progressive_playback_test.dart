import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/gateway_video_playback_port.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_progressive_playback_gateway.dart';

void main() {
  testWidgets('serves remote progressive media from the loopback gateway',
      (tester) async {
    final gateway = FakeProgressivePlaybackGateway();
    final playback = GatewayVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      gateway: gateway,
    );
    final media = VideoMediaSource.remote('https://media.test/clip.mp4');

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: media, isActive: true),
    ));

    expect(find.bySemanticsLabel('Loading video'), findsOneWidget);
    expect(gateway.requests, [media]);

    gateway.completeNext();
    await tester.pump();

    expect(find.text('Progressive loopback stream'), findsOneWidget);
  });
}
