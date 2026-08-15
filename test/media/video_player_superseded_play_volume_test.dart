import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('a mode change during initial play still restores volume', (
    tester,
  ) async {
    final platform = FakeVideoPlayerPlatform()..playBarrier = Completer<void>();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort();
    final media = VideoMediaSource.local('/cache/clip.mp4');
    await pumpVideoPlayerSurface(
      tester,
      port,
      VideoPlaybackSurfaceRequest(media: media, isActive: true),
    );
    expect(platform.calls, contains('play'));

    await pumpVideoPlayerSurface(
      tester,
      port,
      VideoPlaybackSurfaceRequest(
        media: media,
        isActive: true,
        mode: VideoPlaybackMode.accelerated,
      ),
    );
    platform.playBarrier!.complete();
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    final volumeCalls = platform.calls.where(
      (call) => call.startsWith('setVolume:'),
    );
    expect(volumeCalls.last, 'setVolume:1.0');
    expect(platform.calls.where((call) => call == 'play'), hasLength(1));
    expect(platform.playbackSpeeds.last, 2);
  });
}
