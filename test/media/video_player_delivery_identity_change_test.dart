import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/playback_delivery_fixture.dart';
import '../support/recording_playback_telemetry_port.dart';
import '../support/scripted_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('changed explicit delivery identity replaces the surface', (
    tester,
  ) async {
    final platform = ScriptedVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final telemetry = RecordingPlaybackTelemetryPort();
    final port = VideoPlayerPlaybackPort(telemetry: telemetry);
    final media = ProxiedHlsVideoMediaSource(testHlsPlaybackUrl);

    await _show(tester, port, media, 'first');
    await _show(tester, port, media, 'second');

    expect(platform.dataSources, hasLength(2));
    expect(telemetry.activations.map((session) => session.deliveryId.value), [
      'first',
      'second',
    ]);
  });
}

Future<void> _show(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  ProxiedHlsVideoMediaSource media,
  String deliveryId,
) async {
  await tester.pumpWidget(
    MaterialApp(
      home: port.buildSurface(
        VideoPlaybackSurfaceRequest(
          media: media,
          videoId: PlaybackVideoId.parse('clip'),
          isActive: true,
          playbackDeliveryId: PlaybackDeliveryId.parse(deliveryId),
        ),
      ),
    ),
  );
  await tester.pump(const Duration(milliseconds: 100));
  await settleVideoPlayerTasks(tester);
}
