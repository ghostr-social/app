import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';
import '../support/playback_authority_fixture.dart';
import '../support/recording_player_preparation_feedback.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('prepared player sends only its native correlation header', (
    tester,
  ) async {
    final platform = FakeVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort(
      preparationFeedback: RecordingPlayerPreparationFeedback(),
    );
    await pumpVideoPlayerSurface(
      tester,
      port,
      VideoPlaybackSurfaceRequest(
        media: ProxiedProgressiveVideoMediaSource(_playbackUrl),
        isActive: false,
        authority: testPlaybackAuthority(),
      ),
    );

    expect(platform.dataSources.single.httpHeaders, {
      warpPlaybackAttemptHeader: isNotEmpty,
    });
  });
}

const _playbackUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap='
    '$testPlaybackCapability';
