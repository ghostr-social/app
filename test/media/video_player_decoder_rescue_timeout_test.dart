import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/playback_authority_fixture.dart';
import '../support/recovering_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('decoder rescue fails closed when no renewed authority arrives', (
    tester,
  ) async {
    final platform = RecoveringVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort();
    await pumpVideoPlayerSurface(
      tester,
      port,
      VideoPlaybackSurfaceRequest(
        media: ProxiedProgressiveVideoMediaSource(_playbackUrl),
        videoId: PlaybackVideoId.parse('post-1'),
        isActive: true,
        authority: testPlaybackAuthority(),
      ),
    );

    platform.failLatest('[VideoDecoderUnsupported] selected format');
    await tester.pump(const Duration(seconds: 2));
    await tester.pump();

    expect(platform.dataSources, hasLength(1));
    expect(find.text('Video unavailable'), findsOneWidget);
  });
}

const _playbackUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap='
    '$testPlaybackCapability';
