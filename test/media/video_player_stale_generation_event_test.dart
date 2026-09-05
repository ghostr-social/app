import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/recording_playback_telemetry_port.dart';
import '../support/playback_delivery_fixture.dart';
import '../support/playback_authority_fixture.dart';
import '../support/video_player_surface_pump.dart';
import '../support/scripted_video_player_platform.dart';

void main() {
  testWidgets('disposed player events cannot enter the new generation', (
    tester,
  ) async {
    final platform = ScriptedVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final telemetry = RecordingPlaybackTelemetryPort();
    final port = VideoPlayerPlaybackPort(telemetry: telemetry);
    await tester.pumpWidget(
      MaterialApp(
        home: port.buildSurface(
          VideoPlaybackSurfaceRequest(
            media: ProxiedHlsVideoMediaSource(testHlsPlaybackUrl),
            videoId: PlaybackVideoId.parse('clip'),
            isActive: true,
            playbackDeliveryId: testCanonicalPlaybackDeliveryId,
            hlsAuthority: testHlsPlaybackAuthority(),
          ),
        ),
      ),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    platform.emitError('replace');
    await tester.pump();
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    await settleVideoPlayerTasks(tester);
    expect(telemetry.activations, hasLength(2));
    final replacement = telemetry.activations.last;
    final count = telemetry.observations.length;

    platform.emitFor(0, VideoEvent(eventType: VideoEventType.bufferingStart));
    await tester.pump();

    expect(telemetry.observations, hasLength(count));
    expect(telemetry.observations.last.session, replacement);
  });
}
