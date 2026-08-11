import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/recording_playback_telemetry_port.dart';
import '../support/scripted_video_player_platform.dart';

void main() {
  testWidgets('inactive surfaces end their session and ignore later values', (
    tester,
  ) async {
    final platform = ScriptedVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final telemetry = RecordingPlaybackTelemetryPort();
    final port = VideoPlayerPlaybackPort(telemetry: telemetry);
    final media = VideoMediaSource.local('/cache/clip.mp4');
    final id = PlaybackVideoId.parse('clip');

    await pumpSurface(tester, port, surfaceRequest(media, id, true));
    final firstSession = telemetry.activations.single;
    await pumpSurface(tester, port, surfaceRequest(media, id, false));

    expect(telemetry.observations.last.phase, PlaybackPhase.inactive);
    expect(telemetry.deactivations, [firstSession]);
    final inactiveCount = telemetry.observations.length;
    platform.emit(VideoEvent(eventType: VideoEventType.bufferingStart));
    await tester.pump();
    expect(telemetry.observations, hasLength(inactiveCount));

    await pumpSurface(tester, port, surfaceRequest(media, id, true));
    expect(telemetry.activations.last.generation, greaterThan(1));
    expect(telemetry.activations.last, isNot(firstSession));
  });
}

Future<void> pumpSurface(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  VideoPlaybackSurfaceRequest request,
) async {
  await tester.pumpWidget(
    MaterialApp(
      home: port.buildSurface(
        media: request.media,
        videoId: request.videoId,
        isActive: request.isActive,
      ),
    ),
  );
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}

VideoPlaybackSurfaceRequest surfaceRequest(
  VideoMediaSource media,
  PlaybackVideoId id,
  bool isActive,
) {
  return VideoPlaybackSurfaceRequest(
    media: media,
    videoId: id,
    isActive: isActive,
  );
}
