import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/late_video_player_command_platform.dart';
import '../support/video_player_handoff_scene.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('a late superseded play cannot restart paused playback', (
    tester,
  ) async {
    final platform = LateVideoPlayerCommandPlatform();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort();
    await pumpVideoPlayerHandoffScene(tester, port, (
      active: 'a',
      mode: VideoPlaybackMode.normal,
    ));
    platform.blockNext(LateVideoPlayerCommand.play);
    addTearDown(platform.release);

    await pumpVideoPlayerHandoffScene(tester, port, (
      active: 'b',
      mode: VideoPlaybackMode.normal,
    ));
    await platform.entered;
    await pumpVideoPlayerHandoffScene(tester, port, (
      active: 'b',
      mode: VideoPlaybackMode.paused,
    ));
    platform.release();
    await settleVideoPlayerTasks(tester);

    final paused = platform.latestPlayerFor('/cache/b.mp4');
    final isPlaying = platform.isPlaying(paused);
    final volume = platform.volumeFor(paused);
    final audibleOverlap = platform.audibleOverlap;
    final bCreations = platform.creationsFor('/cache/b.mp4');
    await tester.pumpWidget(const SizedBox());
    await settleVideoPlayerTasks(tester);
    expect(isPlaying, isFalse);
    expect(volume, 0);
    expect(audibleOverlap, isFalse);
    expect(bCreations, 2);
  });
}
