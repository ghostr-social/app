import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/hls_video_playback_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/blocking_video_controller_disposer.dart';
import '../support/fake_hls_playback_gateway.dart';
import '../support/fake_video_player_platform.dart';

void main() {
  testWidgets('holds the HLS lease until player teardown completes',
      (tester) async {
    final platform = FakeVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final disposer = BlockingVideoControllerDisposer();
    final gateway = FakeHlsPlaybackGateway();
    final playback = HlsVideoPlaybackPort(
      delegate: VideoPlayerPlaybackPort(controllerDisposer: disposer.call),
      gateway: gateway,
    );
    final media = VideoMediaSource.remote(
      'https://media.test/root.m3u8',
      delivery: VideoMediaDelivery.hls,
    );

    await tester.pumpWidget(MaterialApp(
      home: playback.buildSurface(media: media, isActive: true),
    ));
    gateway.completeNext();
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    expect(gateway.activeLeaseCount, 1);

    await tester.pumpWidget(const MaterialApp(home: SizedBox()));
    expect(disposer.started.isCompleted, isTrue);
    expect(gateway.activeLeaseCount, 1);

    disposer.release.complete();
    for (var index = 0; index < 5; index += 1) {
      await tester.pump();
    }
    expect(gateway.activeLeaseCount, 0);
  });
}
