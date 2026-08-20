import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player/video_player.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/audited_video_player_platform.dart';

void main() {
  testWidgets('unproven native teardown retains media authority', (
    tester,
  ) async {
    final platform = AuditedVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    var releases = 0;
    final port = VideoPlayerPlaybackPort(
      controllerDisposer: (VideoPlayerController _) async {
        throw StateError('injected teardown failure');
      },
    );
    final request = VideoPlaybackSurfaceRequest(
      media: VideoMediaSource.local('/cache/video.mp4'),
      isActive: false,
      onPlaybackMediaReleased: () => releases += 1,
    );

    await tester.pumpWidget(MaterialApp(home: port.buildSurface(request)));
    await tester.pump(const Duration(milliseconds: 100));
    await tester.pumpWidget(const SizedBox());
    await tester.runAsync(() => Future<void>.delayed(Duration.zero));
    await tester.pump();

    expect(platform.playerCount, 1);
    expect(releases, 0);
  });
}
