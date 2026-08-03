import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/hls_video_playback_port.dart';

import '../support/fake_hls_playback_gateway.dart';
import '../support/fake_media_ports.dart';

void main() {
  testWidgets('passes progressive media through without a gateway session',
      (tester) async {
    final gateway = FakeHlsPlaybackGateway();
    final playback = HlsVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      gateway: gateway,
    );
    final media = VideoMediaSource.local('/cache/video.mp4');
    var releases = 0;

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(
        media: media,
        isActive: true,
        onPlaybackMediaReleased: () => releases += 1,
      ),
    ));

    expect(find.text('/cache/video.mp4'), findsOneWidget);
    expect(gateway.requests, isEmpty);

    await tester.pumpWidget(const SizedBox());
    expect(releases, 1);
  });
}
