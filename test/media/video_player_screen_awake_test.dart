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
  testWidgets('screen stays awake exactly while the surface plays', (
    tester,
  ) async {
    final platform = ScriptedVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final screen = RecordingScreenAwakePort();
    final port = VideoPlayerPlaybackPort(
      screenAwake: PlaybackScreenAwakeCoordinator(screen),
    );

    await pumpVideoPlayerSurface(
      tester,
      port,
      VideoPlaybackSurfaceRequest(
        media: VideoMediaSource.local('/cache/clip.mp4'),
        videoId: PlaybackVideoId.parse('clip'),
        isActive: true,
      ),
    );
    expect(screen.toggles, [true]);

    await _advancePlayback(tester, platform);
    expect(screen.toggles, [true]);

    _emitPlaying(platform, false);
    await tester.pump();
    expect(screen.toggles, [true, false]);

    _emitPlaying(platform, true);
    await tester.pump();
    expect(screen.toggles, [true, false, true]);

    await tester.pumpWidget(const MaterialApp(home: SizedBox.shrink()));
    expect(screen.toggles, [true, false, true, false]);
  });
}

Future<void> _advancePlayback(
  WidgetTester tester,
  ScriptedVideoPlayerPlatform platform,
) async {
  _emitPlaying(platform, true);
  await tester.pump();
  platform.position = const Duration(seconds: 1);
  await tester.pump(const Duration(milliseconds: 100));
}

void _emitPlaying(ScriptedVideoPlayerPlatform platform, bool isPlaying) {
  platform.emit(
    VideoEvent(
      eventType: VideoEventType.isPlayingStateUpdate,
      isPlaying: isPlaying,
    ),
  );
}
