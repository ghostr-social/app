import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/hls_video_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import '../support/fake_hls_playback_gateway.dart';
import '../support/fake_media_ports.dart';

void main() {
  testWidgets('releases a stale session when the HLS source changes', (
    tester,
  ) async {
    final gateway = FakeHlsPlaybackGateway();
    final playback = HlsVideoPlaybackPort(
      delegate: FakeVideoPlaybackPort(),
      gateway: gateway,
    );
    VideoMediaSource hls(String host) => VideoMediaSource.remote(
      'https://$host/root.m3u8',
      delivery: VideoMediaDelivery.hls,
    );

    await tester.pumpWidget(
      MaterialApp(
        home: playback.buildSurface(
          VideoPlaybackSurfaceRequest(media: hls('first.test'), isActive: true),
        ),
      ),
    );
    await tester.pumpWidget(
      MaterialApp(
        home: playback.buildSurface(
          VideoPlaybackSurfaceRequest(
            media: hls('second.test'),
            isActive: true,
          ),
        ),
      ),
    );
    expect(gateway.requests, hasLength(2));

    gateway.completeAt(0);
    await tester.pump();
    expect(gateway.activeLeaseCount, 0);
    expect(find.bySemanticsLabel('Loading video'), findsOneWidget);

    gateway.completeAt(1);
    await tester.pump();
    expect(gateway.activeLeaseCount, 1);
  });
}
