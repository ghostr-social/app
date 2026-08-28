import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/failing_rendered_first_frame_port.dart';
import '../support/fake_video_player_platform.dart';
import '../support/playback_delivery_fixture.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('frame release failure cannot exhaust controller capacity', (
    tester,
  ) async {
    final platform = FakeVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final frames = ThrowingReleaseRenderedFirstFramePort();
    final port = VideoPlayerPlaybackPort(renderedFirstFrames: frames);

    for (var index = 0; index < 3; index += 1) {
      await _show(tester, port, index);
      await tester.pumpWidget(const SizedBox());
      await settleVideoPlayerTasks(tester);
    }

    expect(frames.releases, 3);
    expect(platform.dataSources, hasLength(3));
    expect(platform.calls.where((call) => call == 'dispose'), hasLength(3));
  });
}

Future<void> _show(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  int index,
) async {
  await tester.pumpWidget(
    MaterialApp(
      home: port.buildSurface(
        VideoPlaybackSurfaceRequest(
          media: ProxiedHlsVideoMediaSource(testHlsPlaybackUrl),
          videoId: PlaybackVideoId.parse('clip-$index'),
          isActive: true,
          playbackDeliveryId: PlaybackDeliveryId.parse('post-$index'),
        ),
      ),
    ),
  );
  await tester.pump(const Duration(milliseconds: 100));
  await settleVideoPlayerTasks(tester);
}
