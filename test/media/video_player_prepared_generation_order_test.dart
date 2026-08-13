import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/audited_video_player_platform.dart';
import '../support/playback_delivery_fixture.dart';
import '../support/recording_playback_telemetry_port.dart';

void main() {
  testWidgets('prepared predecessor receives a newer activation generation', (
    tester,
  ) async {
    VideoPlayerPlatform.instance = AuditedVideoPlayerPlatform();
    final telemetry = RecordingPlaybackTelemetryPort();
    final port = VideoPlayerPlaybackPort(telemetry: telemetry);

    await showPair(tester, port, activeIndex: 1);
    final current = telemetry.activations.single;
    await showPair(tester, port, activeIndex: 0);

    expect(
      telemetry.activations.last.generation,
      greaterThan(current.generation),
    );
  });
}

Future<void> showPair(
  WidgetTester tester,
  VideoPlayerPlaybackPort port, {
  required int activeIndex,
}) async {
  await tester.pumpWidget(
    MaterialApp(
      home: Stack(
        children: List.generate(2, (index) {
          return port.buildSurface(
            VideoPlaybackSurfaceRequest(
              media: ProxiedHlsVideoMediaSource(testHlsPlaybackUrl),
              videoId: PlaybackVideoId.parse('clip-$index'),
              isActive: index == activeIndex,
            ),
          );
        }),
      ),
    ),
  );
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}
