import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('local controllers do not reserve native correlation tokens', (
    tester,
  ) async {
    VideoPlayerPlatform.instance = FakeVideoPlayerPlatform();
    final frames = _CountingRenderedFrames();
    final port = VideoPlayerPlaybackPort(renderedFirstFrames: frames);

    await pumpVideoPlayerSurface(
      tester,
      port,
      VideoPlaybackSurfaceRequest(
        media: VideoMediaSource.local('/tmp/video.mp4'),
        isActive: false,
      ),
    );

    expect(frames.attempts, 0);
  });
}

final class _CountingRenderedFrames implements RenderedFirstFramePort {
  var attempts = 0;

  @override
  RenderedFirstFrameAttempt? beginAttempt() {
    attempts += 1;
    return null;
  }
}
