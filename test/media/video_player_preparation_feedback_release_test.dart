import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/playback_authority_fixture.dart';
import '../support/recording_player_preparation_feedback.dart';
import '../support/scripted_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('disposing an initialized controller releases its evidence', (
    tester,
  ) async {
    VideoPlayerPlatform.instance = ScriptedVideoPlayerPlatform();
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
    await tester.pumpWidget(const MaterialApp(home: SizedBox.shrink()));
    await settleVideoPlayerTasks(tester);

    expect(feedback.events.map((event) => event.state), [
      RecordedPreparationState.initializing,
      RecordedPreparationState.initialized,
      RecordedPreparationState.released,
    ]);
  });
}

const _playbackUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap='
    '$testPlaybackCapability';
