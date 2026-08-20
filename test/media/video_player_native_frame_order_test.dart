import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';
import 'package:ghostr/platform/media/ffi_player_preparation_feedback_port.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/drain_test_microtasks.dart';
import '../support/fake_video_player_platform.dart';
import '../support/playback_authority_fixture.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('native output waits for plugin and handoff settlement', (
    tester,
  ) async {
    final pause = Completer<void>();
    VideoPlayerPlatform.instance = FakeVideoPlayerPlatform()
      ..pauseBarrier = pause;
    final events = StreamController<Object?>();
    final frames = NativeRenderedFirstFramePort(events: events.stream);
    final sent = <FfiPlayerPreparationReport>[];
    final token = PlayerPreparationAttemptToken.parse(_token);
    final feedback = FfiPlayerPreparationFeedbackPort(
      reportPreparation: ({required input}) async => sent.add(input),
      playerCapabilityGeneration: BigInt.one,
      clientEpoch: BigInt.one,
      monotonicMicros: () => 1,
      tokenFactory: () => token,
    );
    final port = VideoPlayerPlaybackPort(
      preparationFeedback: feedback,
      renderedFirstFrames: frames,
    );
    await pumpVideoPlayerSurface(tester, port, _request());
    events.add({'version': 1, 'attemptToken': _token});
    await tester.runAsync(drainTestMicrotasks);
    expect(sent.map((item) => item.state), [
      FfiPlayerPreparationState.initializing,
    ]);

    pause.complete();
    await settleVideoPlayerTasks(tester);
    await tester.runAsync(drainTestMicrotasks);
    expect(sent.map((item) => item.state), [
      FfiPlayerPreparationState.initializing,
      FfiPlayerPreparationState.initialized,
      FfiPlayerPreparationState.firstFrameRendered,
    ]);
  });
}

VideoPlaybackSurfaceRequest _request() => VideoPlaybackSurfaceRequest(
  media: ProxiedProgressiveVideoMediaSource(
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap='
    '$testPlaybackCapability',
  ),
  isActive: false,
  authority: testPlaybackAuthority(),
);

const _token = 'abcdefghijklmnopqrstuA';
