import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/failing_rendered_first_frame_port.dart';
import '../support/fake_video_player_platform.dart';
import '../support/playback_authority_fixture.dart';
import '../support/recording_player_preparation_feedback.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('failed frame claim settles preparation and releases capacity', (
    tester,
  ) async {
    final platform = FakeVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final frames = ThrowingClaimRenderedFirstFramePort();
    final feedback = RecordingPlayerPreparationFeedback();
    final port = VideoPlayerPlaybackPort(
      preparationFeedback: feedback,
      renderedFirstFrames: frames,
    );

    for (var index = 0; index < 3; index += 1) {
      await _show(tester, port, index);
      expect(find.bySemanticsLabel('Video unavailable'), findsOneWidget);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      await settleVideoPlayerTasks(tester);
    }

    expect(frames.attempts, 3);
    expect(frames.releases, 3);
    expect(platform.dataSources, isEmpty);
    expect(feedback.events.map((event) => event.state), [
      for (var index = 0; index < 3; index += 1) ...[
        RecordedPreparationState.initializing,
        RecordedPreparationState.failed,
      ],
    ]);
    expect(
      feedback.events
          .where((event) => event.failure != null)
          .map((event) => event.failure),
      everyElement(PlayerPreparationFailureKind.initialization),
    );
  });
}

Future<void> _show(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  int index,
) async {
  final postId = 'post-$index';
  await pumpVideoPlayerSurface(
    tester,
    port,
    VideoPlaybackSurfaceRequest(
      media: ProxiedProgressiveVideoMediaSource(_playbackUrl(postId)),
      isActive: true,
      authority: testPlaybackAuthority(postId: postId),
    ),
  );
  await settleVideoPlayerTasks(tester);
}

String _playbackUrl(String postId) {
  return 'http://127.0.0.1:3210/video.mp4?id=$postId&cap='
      '$testPlaybackCapability';
}
