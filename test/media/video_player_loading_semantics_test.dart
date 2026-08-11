import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';

void main() {
  testWidgets('announces video initialization while the player is loading', (
    tester,
  ) async {
    VideoPlayerPlatform.instance = FakeVideoPlayerPlatform();

    await tester.pumpWidget(
      MaterialApp(
        home: VideoPlayerPlaybackPort().buildSurface(
          VideoPlaybackSurfaceRequest(
            media: VideoMediaSource.local('/cache/video.mp4'),
            isActive: true,
          ),
        ),
      ),
    );

    expect(find.bySemanticsLabel('Loading video'), findsOneWidget);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
  });
}
