import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_screen_awake_coordinator.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/recording_screen_awake_port.dart';
import '../support/scripted_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('screen sleeps when the surface loses feed focus', (
    tester,
  ) async {
    final platform = ScriptedVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final screen = RecordingScreenAwakePort();
    final port = VideoPlayerPlaybackPort(
      screenAwake: PlaybackScreenAwakeCoordinator(screen),
    );

    await pumpVideoPlayerSurface(tester, port, _request(isActive: true));
    expect(screen.toggles, [true]);

    await tester.pumpWidget(
      MaterialApp(home: port.buildSurface(_request(isActive: false))),
    );
    await tester.pump();
    expect(screen.toggles, [true, false]);
  });
}

VideoPlaybackSurfaceRequest _request({required bool isActive}) {
  return VideoPlaybackSurfaceRequest(
    media: VideoMediaSource.local('/cache/clip.mp4'),
    videoId: PlaybackVideoId.parse('clip'),
    isActive: isActive,
  );
}
