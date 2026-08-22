import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/feed_preparation_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('superseding a blocked mute lets current handoff continue', (
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
    await _show(tester, port, (active: 'b', mode: VideoPlaybackMode.normal));
    await settleVideoPlayerTasks(tester);

    final current = platform.playerFor('/cache/b.mp4');
    expect(platform.isPlaying(current), isTrue);
    expect(platform.audibleOverlap, isFalse);
    platform.releaseMute();
    await tester.pumpWidget(const SizedBox());
    await settleVideoPlayerTasks(tester);
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
