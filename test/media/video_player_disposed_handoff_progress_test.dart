import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/audited_video_player_platform.dart';

void main() {
  testWidgets('proven disposal releases a blocked playback handoff', (
    tester,
  ) async {
    final platform = AuditedVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final pause = platform.pauseGate = Completer<void>();
    addTearDown(() {
      if (!pause.isCompleted) pause.complete();
    });
    final port = VideoPlayerPlaybackPort();

    await _show(tester, port, 'a', VideoPlaybackMode.normal);
    await _show(tester, port, 'a', VideoPlaybackMode.paused);
    await tester.pumpWidget(const SizedBox());
    await tester.runAsync(() => Future<void>.delayed(Duration.zero));
    await tester.pump();
    expect(platform.playerCount, 0);

    await _show(tester, port, 'b', VideoPlaybackMode.normal);
    expect(platform.isPlaying(1), isTrue, reason: '${platform.commands}');
  });
}

Future<void> _show(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  String id,
  VideoPlaybackMode mode,
) async {
  await tester.pumpWidget(
    MaterialApp(
      home: port.buildSurface(
        VideoPlaybackSurfaceRequest(
          media: VideoMediaSource.local('/cache/$id.mp4'),
          isActive: true,
          mode: mode,
        ),
      ),
    ),
  );
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}
