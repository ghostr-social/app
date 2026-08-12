import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/scripted_video_player_platform.dart';

void main() {
  testWidgets('rejects zero-size media without starting its audio', (
    tester,
  ) async {
    final platform = ScriptedVideoPlayerPlatform(initializedSize: Size.zero);
    VideoPlayerPlatform.instance = platform;

    await tester.pumpWidget(
      MaterialApp(
        home: VideoPlayerPlaybackPort().buildSurface(
          VideoPlaybackSurfaceRequest(
            media: VideoMediaSource.local('/cache/audio-only.mp4'),
            isActive: true,
          ),
        ),
      ),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(find.text('Video unavailable'), findsOneWidget);
    expect(platform.playCalls, 0);
  });
}
