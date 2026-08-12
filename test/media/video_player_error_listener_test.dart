import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player/video_player.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/scripted_video_player_platform.dart';

void main() {
  testWidgets('shows the error panel when playback fails mid-stream', (
    tester,
  ) async {
    final platform = ScriptedVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;

    await tester.pumpWidget(
      MaterialApp(
        home:
            VideoPlayerPlaybackPort(
              recoveryPolicy: PlaybackRecoveryPolicy.disabled(),
            ).buildSurface(
              VideoPlaybackSurfaceRequest(
                media: VideoMediaSource.local('/cache/video.mp4'),
                isActive: true,
              ),
            ),
      ),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    expect(find.byType(VideoPlayer), findsOneWidget);

    platform.emitError('Stream reset by the host');
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(find.byType(VideoPlayer), findsNothing);
    expect(find.text('Video unavailable'), findsOneWidget);
    expect(find.text('Retry'), findsOneWidget);
  });
}
