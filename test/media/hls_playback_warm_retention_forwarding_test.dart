import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/hls_video_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import '../support/fake_hls_playback_gateway.dart';
import '../support/fake_media_ports.dart';

void main() {
  testWidgets('HLS gateway preserves warm inactive playback intent', (
    tester,
  ) async {
    final delegate = FakeVideoPlaybackPort();
    final gateway = FakeHlsPlaybackGateway();
    final playback = HlsVideoPlaybackPort(delegate: delegate, gateway: gateway);
    final media = VideoMediaSource.remote(
      'https://media.test/root.m3u8',
      delivery: VideoMediaDelivery.hls,
    );

    await tester.pumpWidget(
      MaterialApp(
        home: playback.buildSurface(
          VideoPlaybackSurfaceRequest(
            media: media,
            isActive: false,
            keepWarmWhenInactive: true,
          ),
        ),
      ),
    );
    gateway.completeNext();
    await tester.pump();

    expect(delegate.requests.single.keepWarmWhenInactive, isTrue);
  });
}
