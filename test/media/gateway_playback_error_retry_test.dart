import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/gateway_video_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_progressive_playback_gateway.dart';

void main() {
  testWidgets('offers retry after gateway resolution fails', (tester) async {
    final gateway = FakeProgressivePlaybackGateway();
    final playback = GatewayVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      gateway: gateway,
    );
    final media = VideoMediaSource.remote('https://media.test/clip.mp4');

    await tester.pumpWidget(
      MaterialApp(
        home: playback.buildSurface(
          VideoPlaybackSurfaceRequest(media: media, isActive: true),
        ),
      ),
    );
    gateway.failNext();
    await tester.pump();

    expect(find.text('Video unavailable'), findsOneWidget);

    await tester.tap(find.text('Retry'));
    await tester.pump();
    expect(gateway.requests, hasLength(2));

    gateway.completeNext();
    await tester.pump();
    expect(find.text('Progressive loopback stream'), findsOneWidget);
  });
}
