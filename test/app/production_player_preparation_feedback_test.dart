import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_playback.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/rendered_first_frame_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_remote_video_source.dart';
import '../support/fake_video_player_platform.dart';
import '../support/playback_authority_fixture.dart';
import '../support/recording_player_preparation_feedback.dart';
import '../support/test_video_delivery.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('production player reports prepared progressive lifecycle', (
    tester,
  ) async {
    VideoPlayerPlatform.instance = FakeVideoPlayerPlatform();
    final feedback = RecordingPlayerPreparationFeedback();
    final playback = buildProductionVideoPlayback(
      testVideoDelivery(remoteSource: FakeRemoteVideoSource([])),
      playerPreparationFeedback: feedback,
      renderedFirstFrames: const NoopRenderedFirstFramePort(),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: playback.buildSurface(
          VideoPlaybackSurfaceRequest(
            media: ProxiedProgressiveVideoMediaSource(_playbackUrl),
            isActive: false,
            authority: testPlaybackAuthority(),
          ),
        ),
      ),
    );
    await tester.pump(const Duration(milliseconds: 100));
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
