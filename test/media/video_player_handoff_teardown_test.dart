import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/audited_video_player_platform.dart';

void main() {
  testWidgets('a blocked handoff cannot retain a closing decoder', (
    tester,
  ) async {
    final platform = AuditedVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final gate = platform.pauseGate = Completer<void>();
    addTearDown(() {
      if (!gate.isCompleted) gate.complete();
    });
    final port = VideoPlayerPlaybackPort();
    final media = VideoMediaSource.local('/cache/video.mp4');
    var releases = 0;

    final normal = VideoPlaybackSurfaceRequest(
      media: media,
      isActive: true,
      onPlaybackMediaReleased: () => releases += 1,
    );
    final paused = VideoPlaybackSurfaceRequest(
      media: media,
      isActive: true,
      mode: VideoPlaybackMode.paused,
      onPlaybackMediaReleased: () => releases += 1,
    );
    await _show(tester, port, normal);
    await _show(tester, port, paused);
    expect(platform.commands, contains('pause:0'));
    await tester.pumpWidget(const SizedBox());
    await tester.runAsync(() => Future<void>.delayed(Duration.zero));
    await tester.pump();

    expect(platform.playerCount, 0);
    expect(releases, 1);
  });
}

Future<void> _show(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  VideoPlaybackSurfaceRequest request,
) async {
  await tester.pumpWidget(MaterialApp(home: port.buildSurface(request)));
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}
