import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/rendered_first_frame_protocol.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';
import '../support/playback_authority_fixture.dart';
import '../support/recording_playback_telemetry_port.dart';
import '../support/recording_player_preparation_feedback.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('a disposed attempt cannot report its scheduled presentation', (
    tester,
  ) async {
    final platform = FakeVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final events = StreamController<Object?>();
    final telemetry = RecordingPlaybackTelemetryPort();
    final port = VideoPlayerPlaybackPort(
      telemetry: telemetry,
      preparationFeedback: RecordingPlayerPreparationFeedback(),
      renderedFirstFrames: NativeRenderedFirstFramePort(events: events.stream),
    );
    await pumpVideoPlayerSurface(tester, port, request(false));
    await settleVideoPlayerTasks(tester);
    final token =
        platform.dataSources.single.httpHeaders[warpPlaybackAttemptHeader]!;
    events.add({'version': 1, 'attemptToken': token});
    await tester.runAsync(() => Future<void>.delayed(Duration.zero));
    await tester.pumpWidget(
      MaterialApp(home: port.buildSurface(request(true))),
    );
    await tester.runAsync(() => Future<void>.delayed(Duration.zero));

    await tester.pumpWidget(const SizedBox());

    expect(telemetry.presentations, isEmpty);
  });
}

VideoPlaybackSurfaceRequest request(bool active) => VideoPlaybackSurfaceRequest(
  media: ProxiedProgressiveVideoMediaSource(
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap=$testPlaybackCapability',
  ),
  videoId: PlaybackVideoId.parse('clip'),
  authority: testPlaybackAuthority(),
  isActive: active,
);
