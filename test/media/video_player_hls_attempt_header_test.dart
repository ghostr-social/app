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
  testWidgets('prepared HLS player sends its native correlation header', (
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
        media: ProxiedHlsVideoMediaSource(_playbackUrl),
        isActive: false,
        authority: testPlaybackAuthority(postId: _sessionId),
      ),
    );

    expect(platform.dataSources.single.httpHeaders, {
      warpPlaybackAttemptHeader: isNotEmpty,
    });
  });
}

const _sessionId =
    '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';
const _playbackUrl = 'http://127.0.0.1:3210/hls/$_sessionId/index.m3u8';
