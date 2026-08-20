import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/recording_playback_telemetry_port.dart';
import '../support/playback_delivery_fixture.dart';
import '../support/scripted_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('every native replacement gets one delivery-bound generation', (
    tester,
  ) async {
    final platform = ScriptedVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final telemetry = RecordingPlaybackTelemetryPort();
    var port = VideoPlayerPlaybackPort(telemetry: telemetry);
    final media = ProxiedHlsVideoMediaSource(testHlsPlaybackUrl);
    final request = VideoPlaybackSurfaceRequest(
      media: media,
      videoId: PlaybackVideoId.parse('clip'),
      isActive: true,
    );

    await _pump(tester, port, request);
    final first = telemetry.activations.single;
    expect(first.deliveryId, PlaybackDeliveryId.parse(testPlaybackDeliveryId));
    platform.emit(VideoEvent(eventType: VideoEventType.bufferingStart));
    await tester.pump();
    expect(
      telemetry.observations.every((item) => item.session == first),
      isTrue,
    );

    await _pump(tester, port, _activity(request, false));
    port = VideoPlayerPlaybackPort(telemetry: telemetry);
    await _pump(tester, port, request);
    final second = telemetry.activations.last;
    expect(second.generation, greaterThan(first.generation));
    expect(second.deliveryId, first.deliveryId);

    platform.emitError('replace this player');
    await tester.pump();
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    await settleVideoPlayerTasks(tester);
    expect(
      telemetry.activations.last.generation,
      greaterThan(second.generation),
    );
    expect(telemetry.activations.last.deliveryId, first.deliveryId);
  });
}

Future<void> _pump(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  VideoPlaybackSurfaceRequest request,
) async {
  await tester.pumpWidget(MaterialApp(home: port.buildSurface(request)));
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
  await settleVideoPlayerTasks(tester);
}

VideoPlaybackSurfaceRequest _activity(
  VideoPlaybackSurfaceRequest request,
  bool isActive,
) {
  return VideoPlaybackSurfaceRequest(
    media: request.media,
    videoId: request.videoId,
    isActive: isActive,
  );
}
