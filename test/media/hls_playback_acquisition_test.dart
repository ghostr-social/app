import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/hls_video_playback_port.dart';

import '../support/fake_hls_playback_gateway.dart';
import '../support/fake_media_ports.dart';

void main() {
  testWidgets('renders the trusted proxy after gateway acquisition',
      (tester) async {
    final gateway = FakeHlsPlaybackGateway();
    final playback = HlsVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      gateway: gateway,
    );
    final media = VideoMediaSource.remote(
      'https://media.test/root.m3u8',
      delivery: VideoMediaDelivery.hls,
    );
    var upstreamReleases = 0;

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(
        media: media,
        isActive: true,
        onPlaybackMediaReleased: () => upstreamReleases += 1,
      ),
    ));
    gateway.completeNext();
    await tester.pump();

    expect(find.text('Secure HLS stream'), findsOneWidget);
    expect(gateway.activeLeaseCount, 1);
    expect(gateway.requests.single.sourceUrls.single.host, 'media.test');

    await tester.pumpWidget(const SizedBox());
    expect(gateway.activeLeaseCount, 0);
    expect(upstreamReleases, 1);
  });
}
