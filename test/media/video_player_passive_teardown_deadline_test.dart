import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/audited_video_player_platform.dart';

void main() {
  testWidgets('passive close does not arm a teardown deadline', (tester) async {
    final platform = AuditedVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    var releases = 0;
    final port = VideoPlayerPlaybackPort(
      controllerDisposer: (_) => Completer<void>().future,
    );

    await tester.pumpWidget(
      MaterialApp(
        home: port.buildSurface(
          VideoPlaybackSurfaceRequest(
            media: VideoMediaSource.local('/cache/passive.mp4'),
            isActive: false,
            onPlaybackMediaReleased: () => releases += 1,
          ),
        ),
      ),
    );
    await tester.pump(const Duration(milliseconds: 100));
    await tester.pumpWidget(const SizedBox.shrink());
    await tester.runAsync(() => Future<void>.delayed(Duration.zero));
    await tester.pump();

    expect(platform.playerCount, 1);
    expect(releases, 0);
  });
}
