import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/feed_preparation_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('a late superseded mute cannot silence current playback', (
    tester,
  ) async {
    final platform = FeedPreparationVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort();
    await _show(tester, port, (active: 'a', mode: VideoPlaybackMode.normal));
    platform.blockNextMute();
    addTearDown(platform.releaseMute);

    await _show(tester, port, (active: 'b', mode: VideoPlaybackMode.normal));
    await platform.muteEntered;
    await _show(tester, port, (active: 'b', mode: VideoPlaybackMode.paused));
    platform.releaseMute();
    await settleVideoPlayerTasks(tester);

    final current = platform.playerFor('/cache/a.mp4');
    final isPlaying = platform.isPlaying(current);
    final volume = platform.volumeFor(current);
    final audibleOverlap = platform.audibleOverlap;
    await tester.pumpWidget(const SizedBox());
    await settleVideoPlayerTasks(tester);
    expect(isPlaying, isTrue);
    expect(volume, 1);
    expect(audibleOverlap, isFalse);
  });
}

Future<void> _show(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  ({String active, VideoPlaybackMode mode}) scene,
) async {
  await tester.pumpWidget(
    MaterialApp(
      home: Stack(
        children: ['a', 'b'].map((id) {
          return port.buildSurface(
            VideoPlaybackSurfaceRequest(
              media: VideoMediaSource.local('/cache/$id.mp4'),
              isActive: id == 'a' || id == scene.active,
              mode: id == scene.active ? scene.mode : VideoPlaybackMode.normal,
            ),
          );
        }).toList(),
      ),
    ),
  );
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}
