import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';
import '../support/playback_authority_fixture.dart';
import '../support/recording_playback_telemetry_port.dart';
import '../support/recording_player_preparation_feedback.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('warm reactivation presents a new session without reload', (
    tester,
  ) async {
    final platform = FakeVideoPlayerPlatform();
    final frames = StreamController<Object?>();
    final telemetry = RecordingPlaybackTelemetryPort();
    final preparation = RecordingPlayerPreparationFeedback();
    final scope = VideoPlaybackSurfaceScope();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort(
      telemetry: telemetry,
      preparationFeedback: preparation,
      renderedFirstFrames: NativeRenderedFirstFramePort(events: frames.stream),
    );
    await pumpVideoPlayerSurface(tester, port, _request(scope, true));
    await settleVideoPlayerTasks(tester);
    final token = platform.dataSources.single
        .httpHeaders[warpPlaybackAttemptHeader]!;
    frames.add({'version': 1, 'attemptToken': token});
    await settleVideoPlayerTasks(tester);

    await tester.pumpWidget(
      MaterialApp(home: port.buildSurface(_request(scope, false))),
    );
    await settleVideoPlayerTasks(tester);
    await tester.pumpWidget(
      MaterialApp(home: port.buildSurface(_request(scope, true))),
    );
    await settleVideoPlayerTasks(tester);

    expect(platform.dataSources, hasLength(1));
    expect(telemetry.activations, hasLength(2));
    expect(telemetry.activations.toSet(), hasLength(2));
    expect(telemetry.presentations, telemetry.activations);
    expect(
      preparation.events.where(
        (event) => event.state == RecordedPreparationState.released,
      ),
      isEmpty,
    );
  });
}

VideoPlaybackSurfaceRequest _request(
  VideoPlaybackSurfaceScope scope,
  bool active,
) => VideoPlaybackSurfaceRequest(
  media: ProxiedProgressiveVideoMediaSource(
    'http://127.0.0.1:3210/video.mp4?id=warm&cap=$testPlaybackCapability',
  ),
  videoId: PlaybackVideoId.parse('warm'),
  authority: testPlaybackAuthority(postId: 'warm'),
  isActive: active,
  surfaceScope: scope,
  keepWarmWhenInactive: !active,
);
