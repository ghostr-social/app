import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/gateway_video_playback_port.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_progressive_playback_gateway.dart';

void main() {
  testWidgets('re-resolves the gateway URL when the media changes',
      (tester) async {
    final gateway = FakeProgressivePlaybackGateway();
    final playback = GatewayVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      gateway: gateway,
    );
    final first = VideoMediaSource.remote('https://media.test/first.mp4');
    final second = VideoMediaSource.remote('https://media.test/second.mp4');

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: first, isActive: true),
    ));
    gateway.completeNext();
    await tester.pump();
    expect(find.text('Progressive loopback stream'), findsOneWidget);

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: second, isActive: true),
    ));

    expect(gateway.requests, [first, second]);
    expect(find.bySemanticsLabel('Loading video'), findsOneWidget);

    gateway.completeNext(
      playbackUrl: 'http://127.0.0.1:3210/video.mp4?id=post-2',
    );
    await tester.pump();
    expect(find.text('Progressive loopback stream'), findsOneWidget);
  });
}
