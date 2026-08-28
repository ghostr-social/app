import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_screen_awake_coordinator.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/playback_delivery_fixture.dart';
import '../support/recording_playback_telemetry_port.dart';
import '../support/recording_screen_awake_port.dart';
import '../support/scripted_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets(
    'an immediate user pause reports paused and releases wakefulness',
    (tester) async {
      VideoPlayerPlatform.instance = ScriptedVideoPlayerPlatform();
      final telemetry = RecordingPlaybackTelemetryPort();
      final screen = RecordingScreenAwakePort();
      final port = VideoPlayerPlaybackPort(
        telemetry: telemetry,
        screenAwake: PlaybackScreenAwakeCoordinator(screen),
      );
      final media = ProxiedHlsVideoMediaSource(testHlsPlaybackUrl);
      await pumpVideoPlayerSurface(
        tester,
        port,
        VideoPlaybackSurfaceRequest(
          media: media,
          videoId: PlaybackVideoId.parse('clip'),
          isActive: true,
          playbackDeliveryId: testCanonicalPlaybackDeliveryId,
        ),
      );

      await pumpVideoPlayerSurface(
        tester,
        port,
        VideoPlaybackSurfaceRequest(
          media: media,
          videoId: PlaybackVideoId.parse('clip'),
          isActive: true,
          mode: VideoPlaybackMode.paused,
          playbackDeliveryId: testCanonicalPlaybackDeliveryId,
        ),
      );

      expect(telemetry.activations, hasLength(1));
      expect(telemetry.deactivations, isEmpty);
      expect(telemetry.observations.last.phase, PlaybackPhase.paused);
      expect(screen.toggles, [true, false]);
    },
  );
}
