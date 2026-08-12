import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/scripted_video_player_platform.dart';

void main() {
  testWidgets('reconstruction clears the stale stall announcement', (
    tester,
  ) async {
    final platform = ScriptedVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort(
      recoveryPolicy: PlaybackRecoveryPolicy([const Duration(seconds: 1)]),
    );
    await tester.pumpWidget(
      MaterialApp(
        home: port.buildSurface(
          VideoPlaybackSurfaceRequest(
            media: VideoMediaSource.local('/cache/video.mp4'),
            isActive: true,
          ),
        ),
      ),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    platform.position = const Duration(seconds: 1);
    await tester.pump(const Duration(milliseconds: 100));
    platform.emit(
      VideoEvent(
        eventType: VideoEventType.isPlayingStateUpdate,
        isPlaying: true,
      ),
    );
    platform.emit(VideoEvent(eventType: VideoEventType.bufferingStart));
    platform.emit(
      VideoEvent(
        eventType: VideoEventType.isPlayingStateUpdate,
        isPlaying: false,
      ),
    );
    await tester.pump();
    await tester.pump();
    expect(find.bySemanticsLabel('Buffering video'), findsOneWidget);

    platform.emitError('stalled connection closed');
    await tester.pump();
    await tester.pump();

    expect(find.bySemanticsLabel('Buffering video'), findsNothing);
    expect(find.bySemanticsLabel('Loading video'), findsOneWidget);
  });
}
