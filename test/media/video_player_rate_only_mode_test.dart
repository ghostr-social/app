import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('speed-only modes do not restart or mute active playback', (
    tester,
  ) async {
    final platform = FakeVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort();
    final media = VideoMediaSource.local('/cache/clip.mp4');
    await pumpVideoPlayerSurface(
      tester,
      port,
      VideoPlaybackSurfaceRequest(media: media, isActive: true),
    );
    platform.calls.clear();
    platform.playbackSpeeds.clear();

    await pumpVideoPlayerSurface(
      tester,
      port,
      VideoPlaybackSurfaceRequest(
        media: media,
        isActive: true,
        mode: VideoPlaybackMode.accelerated,
      ),
    );
    await pumpVideoPlayerSurface(
      tester,
      port,
      VideoPlaybackSurfaceRequest(media: media, isActive: true),
    );

    expect(platform.playbackSpeeds, [2, 1]);
    expect(platform.calls, isNot(contains('play')));
    expect(platform.calls, isNot(contains(startsWith('setVolume:'))));
  });
}
