import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/playback_authority_fixture.dart';
import '../support/recording_playback_telemetry_port.dart';
import '../support/scripted_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('mismatched explicit delivery identity fails closed', (
    tester,
  ) async {
    VideoPlayerPlatform.instance = ScriptedVideoPlayerPlatform();
    final telemetry = RecordingPlaybackTelemetryPort();
    final port = VideoPlayerPlaybackPort(telemetry: telemetry);

    await tester.pumpWidget(
      MaterialApp(
        home: port.buildSurface(
          VideoPlaybackSurfaceRequest(
            media: ProxiedProgressiveVideoMediaSource(_playbackUrl),
            videoId: PlaybackVideoId.parse('clip'),
            isActive: true,
            playbackDeliveryId: PlaybackDeliveryId.parse('wrong-delivery'),
          ),
        ),
      ),
    );
    await tester.pump(const Duration(milliseconds: 100));
    await settleVideoPlayerTasks(tester);

    expect(telemetry.activations, isEmpty);
    expect(telemetry.observations, isEmpty);
  });
}

const _playbackUrl =
    'http://127.0.0.1:3210/video.mp4?id=actual-delivery&cap='
    '$testPlaybackCapability';
