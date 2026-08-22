import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/feed_preparation_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('a late target mute is repaired after reactivation', (
    tester,
  ) async {
    final platform = FeedPreparationVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort();
    await _show(tester, port, VideoPlaybackMode.paused);
    platform.blockNextMute();
    addTearDown(platform.releaseMute);

    await _show(tester, port, VideoPlaybackMode.normal);
    await platform.muteEntered;
    await _show(tester, port, VideoPlaybackMode.paused);
    await _show(tester, port, VideoPlaybackMode.normal);
    platform.releaseMute();
    await settleVideoPlayerTasks(tester);

    final current = platform.playerFor('/cache/a.mp4');
    final volume = platform.volumeFor(current);
    final isPlaying = platform.isPlaying(current);
    await tester.pumpWidget(const SizedBox());
    await settleVideoPlayerTasks(tester);
    expect(volume, 1);
    expect(isPlaying, isTrue);
    expect(platform.audibleOverlap, isFalse);
  });
}

Future<void> _show(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  VideoPlaybackMode mode,
) async {
  await tester.pumpWidget(
    MaterialApp(
      home: port.buildSurface(
        VideoPlaybackSurfaceRequest(
          media: VideoMediaSource.local('/cache/a.mp4'),
          isActive: true,
          mode: mode,
        ),
      ),
    ),
  );
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}
