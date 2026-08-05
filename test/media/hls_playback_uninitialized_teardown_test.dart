import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/hls_video_playback_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_hls_playback_gateway.dart';
import '../support/fake_video_player_platform.dart';

void main() {
  testWidgets('releases an HLS lease when unmounted before initialization', (
    tester,
  ) async {
    final platform = FakeVideoPlayerPlatform(autoInitialize: false);
    VideoPlayerPlatform.instance = platform;
    final gateway = FakeHlsPlaybackGateway();
    var disposalCount = 0;
    final playback = HlsVideoPlaybackPort(
      delegate: VideoPlayerPlaybackPort(
        controllerDisposer: (_) async => disposalCount += 1,
      ),
      gateway: gateway,
    );
    final media = VideoMediaSource.remote(
      'https://media.test/root.m3u8',
      delivery: VideoMediaDelivery.hls,
    );

    await tester.pumpWidget(
      MaterialApp(home: playback.buildSurface(media: media, isActive: true)),
    );
    gateway.completeNext();
    await tester.pump();
    expect(gateway.activeLeaseCount, 1);

    await tester.pumpWidget(const MaterialApp(home: SizedBox()));
    await tester.pump();

    expect(disposalCount, 1);
    expect(gateway.activeLeaseCount, 0);
  });
}
