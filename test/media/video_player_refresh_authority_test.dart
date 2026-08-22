import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/playback_authority_fixture.dart';
import '../support/recording_playback_telemetry_port.dart';
import '../support/recording_player_preparation_feedback.dart';
import '../support/recovering_video_player_platform.dart';
import '../support/scripted_progressive_playback_refresh.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('renewed capability opens its own measured preparation', (
    tester,
  ) async {
    final platform = RecoveringVideoPlayerPlatform();
    final refresh = ScriptedProgressivePlaybackRefresh();
    final feedback = RecordingPlayerPreparationFeedback();
    final frames = StreamController<Object?>();
    final telemetry = RecordingPlaybackTelemetryPort();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort(
      telemetry: telemetry,
      preparationFeedback: feedback,
      renderedFirstFrames: NativeRenderedFirstFramePort(events: frames.stream),
      recoveryPolicy: PlaybackRecoveryPolicy([Duration.zero]),
    );
    await pumpVideoPlayerSurface(
      tester,
      port,
      VideoPlaybackSurfaceRequest(
        media: ProxiedProgressiveVideoMediaSource(_firstUrl),
        videoId: PlaybackVideoId.parse('post-1'),
        isActive: true,
        authority: testPlaybackAuthority(),
        progressiveRefresh: refresh,
      ),
    );
    await settleVideoPlayerTasks(tester);
    final firstToken = platform.dataSources.single
        .httpHeaders[warpPlaybackAttemptHeader];

    platform.failLatest('source interrupted');
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    refresh.completeNext(_secondUrl);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    await settleVideoPlayerTasks(tester);
    final secondToken = platform.dataSources.last
        .httpHeaders[warpPlaybackAttemptHeader];

    expect(feedback.events.map((event) => event.state), [
      RecordedPreparationState.initializing,
      RecordedPreparationState.initialized,
      RecordedPreparationState.failed,
      RecordedPreparationState.initializing,
      RecordedPreparationState.initialized,
    ]);
    expect(firstToken, isNotNull);
    expect(secondToken, isNotNull);
    expect(secondToken, isNot(firstToken));
    expect(feedback.events.last.authority.assetId.value, _secondCapability);
    expect(platform.dataSources, hasLength(2));
    expect(platform.dataSources.last.uri, _secondUrl);
    frames.add({'version': 1, 'attemptToken': secondToken!});
    await settleVideoPlayerTasks(tester);

    expect(
      feedback.events.last.state,
      RecordedPreparationState.firstFrameRendered,
    );
    expect(telemetry.presentations, hasLength(1));
  });
}

const _firstUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap='
    '$testPlaybackCapability';
const _secondCapability = 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
const _secondUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap=$_secondCapability';
