import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/playback_authority_fixture.dart';
import '../support/recording_player_preparation_feedback.dart';
import '../support/recovering_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('decoder incompatibility is permanent capability feedback', (
    tester,
  ) async {
    final platform = RecoveringVideoPlayerPlatform(
      initializationFailures: 1,
      initializationErrorCode: 'VideoDecoderUnsupported',
    );
    VideoPlayerPlatform.instance = platform;
    final feedback = RecordingPlayerPreparationFeedback();
    final port = VideoPlayerPlaybackPort(preparationFeedback: feedback);

    await pumpVideoPlayerSurface(
      tester,
      port,
      VideoPlaybackSurfaceRequest(
        media: ProxiedProgressiveVideoMediaSource(_playbackUrl),
        isActive: true,
        authority: testPlaybackAuthority(),
      ),
    );
    await settleVideoPlayerTasks(tester);

    expect(platform.dataSources, hasLength(1));
    expect(feedback.events.last.state, RecordedPreparationState.failed);
    expect(feedback.events.last.failure?.name, 'decoderUnsupported');
  });
}

const _playbackUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap='
    '$testPlaybackCapability';
