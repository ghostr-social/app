import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/playback_authority_fixture.dart';
import '../support/recording_player_preparation_feedback.dart';
import '../support/scripted_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('zero-size decoded track waits for a replacement capability', (
    tester,
  ) async {
    VideoPlayerPlatform.instance = ScriptedVideoPlayerPlatform(
      initializedSize: Size.zero,
    );
    final feedback = RecordingPlayerPreparationFeedback();
    final port = VideoPlayerPlaybackPort(preparationFeedback: feedback);

    await pumpVideoPlayerSurface(
      tester,
      port,
      VideoPlaybackSurfaceRequest(
        media: ProxiedProgressiveVideoMediaSource(_playbackUrl),
        isActive: false,
        authority: testPlaybackAuthority(),
      ),
    );
    await settleVideoPlayerTasks(tester);

    expect(feedback.events.map((event) => event.state), [
      RecordedPreparationState.initializing,
      RecordedPreparationState.failed,
    ]);
    expect(
      feedback.events.last.failure,
      PlayerPreparationFailureKind.invalidVideoTrack,
    );
    expect(find.bySemanticsLabel('Loading video'), findsOneWidget);
    expect(find.text('Video unavailable'), findsNothing);
  });
}

const _playbackUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap='
    '$testPlaybackCapability';
