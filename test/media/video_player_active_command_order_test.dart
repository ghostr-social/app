import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/scripted_video_player_platform.dart';

void main() {
  testWidgets('rapid active changes finish with the newest playback intent', (
    tester,
  ) async {
    final platform = ScriptedVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    const port = VideoPlayerPlaybackPort();
    final media = VideoMediaSource.local('/cache/clip.mp4');
    await pumpSurface(tester, port, media, isActive: true);
    platform.commands.clear();
    platform.pauseGate = Completer<void>();

    await pumpSurface(tester, port, media, isActive: false, settle: false);
    await pumpSurface(tester, port, media, isActive: true, settle: false);

    expect(platform.commands, ['pause']);
    platform.pauseGate!.complete();
    await tester.pump();
    await tester.pump();
    expect(platform.commands, ['pause', 'play']);
  });
}

Future<void> pumpSurface(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  VideoMediaSource media, {
  required bool isActive,
  bool settle = true,
}) async {
  await tester.pumpWidget(
    MaterialApp(
      home: port.buildSurface(media: media, isActive: isActive),
    ),
  );
  await tester.pump();
  if (settle) await tester.pump(const Duration(milliseconds: 100));
}
