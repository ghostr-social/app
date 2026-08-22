import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';
import '../support/playback_authority_fixture.dart';
import '../support/recording_player_preparation_feedback.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('initialized waits for inactive handoff settlement', (
    tester,
  ) async {
    final pause = Completer<void>();
    final platform = FakeVideoPlayerPlatform()..pauseBarrier = pause;
    VideoPlayerPlatform.instance = platform;
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

    expect(feedback.events.map((event) => event.state), [
      RecordedPreparationState.initializing,
    ]);
    pause.complete();
    await settleVideoPlayerTasks(tester);
    expect(feedback.events.map((event) => event.state), [
      RecordedPreparationState.initializing,
      RecordedPreparationState.initialized,
    ]);
  });
}

const _playbackUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap='
    '$testPlaybackCapability';
