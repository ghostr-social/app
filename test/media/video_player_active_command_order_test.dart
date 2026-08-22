import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/scripted_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('rapid cover and return starts only the replacement player', (
    tester,
  ) async {
    final platform = ScriptedVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort();
    final media = VideoMediaSource.local('/cache/clip.mp4');
    await pumpSurface(tester, port, media, isActive: true);
    platform.commands.clear();

    await pumpSurface(tester, port, media, isActive: false, settle: false);
    await pumpSurface(tester, port, media, isActive: true, settle: false);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    await settleVideoPlayerTasks(tester);

    expect(platform.dataSources, hasLength(2));
    expect(
      platform.commands.where((command) => command == 'play'),
      hasLength(1),
    );
    expect(platform.commands, isNot(contains(startsWith('seek:'))));
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
      home: port.buildSurface(
        VideoPlaybackSurfaceRequest(media: media, isActive: isActive),
      ),
    ),
  );
  await tester.pump();
  if (settle) await tester.pump(const Duration(milliseconds: 100));
}
