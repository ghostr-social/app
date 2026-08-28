import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/failing_rendered_first_frame_port.dart';
import '../support/fake_video_player_platform.dart';
import '../support/playback_delivery_fixture.dart';
import '../support/recording_playback_telemetry_port.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('frame listener failure cannot block playable video', (
    tester,
  ) async {
    final platform = FakeVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final frames = ThrowingListenRenderedFirstFramePort();
    final telemetry = RecordingPlaybackTelemetryPort();
    final port = VideoPlayerPlaybackPort(
      telemetry: telemetry,
      renderedFirstFrames: frames,
    );

    await pumpVideoPlayerSurface(
      tester,
      port,
      VideoPlaybackSurfaceRequest(
        media: ProxiedHlsVideoMediaSource(testHlsPlaybackUrl),
        videoId: PlaybackVideoId.parse('clip'),
        isActive: true,
        playbackDeliveryId: testCanonicalPlaybackDeliveryId,
      ),
    );
    await settleVideoPlayerTasks(tester);

    expect(platform.dataSources, hasLength(1));
    expect(platform.calls, contains('play'));
    expect(telemetry.observations, isNotEmpty);
    expect(telemetry.presentations, isEmpty);
    expect(frames.releases, 1);

    await tester.pumpWidget(const SizedBox());
    await settleVideoPlayerTasks(tester);
    expect(frames.releases, 1);
    expect(platform.calls.where((call) => call == 'dispose'), hasLength(1));
  });
}
