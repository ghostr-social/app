import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/playback_authority_fixture.dart';
import '../support/recording_player_preparation_feedback.dart';
import '../support/recovering_video_player_platform.dart';
import '../support/scripted_progressive_playback_refresh.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('late refresh cannot overwrite a newer retained capability', (
    tester,
  ) async {
    final platform = RecoveringVideoPlayerPlatform();
    final refresh = ScriptedProgressivePlaybackRefresh();
    final feedback = RecordingPlayerPreparationFeedback();
    final disposal = Completer<void>();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort(
      preparationFeedback: feedback,
      recoveryPolicy: PlaybackRecoveryPolicy([Duration.zero, Duration.zero]),
      controllerDisposer: (controller) async {
        await disposal.future;
        await controller.dispose();
      },
    );

    await _pump(tester, port, refresh, true);
    await tester.pump(const Duration(milliseconds: 100));
    platform.failLatest('source interrupted');
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    expect(refresh.requestCount, 1);
    await _pump(tester, port, refresh, false);
    await _pump(tester, port, refresh, true);
    await tester.pump(const Duration(milliseconds: 100));
    expect(refresh.requestCount, 2);

    refresh.completeAt(1, _latestUrl);
    await tester.pump();
    refresh.completeAt(0, _staleUrl);
    await tester.pump();
    disposal.complete();
    await settleVideoPlayerTasks(tester);

    expect(platform.dataSources, hasLength(2));
    expect(platform.dataSources.last.uri, _latestUrl);
    expect(feedback.events.last.authority.assetId.value, _latestCapability);
  });
}

Future<void> _pump(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  ScriptedProgressivePlaybackRefresh refresh,
  bool active,
) {
  return tester.pumpWidget(
    MaterialApp(
      home: port.buildSurface(
        VideoPlaybackSurfaceRequest(
          media: ProxiedProgressiveVideoMediaSource(_firstUrl),
          isActive: active,
          authority: testPlaybackAuthority(),
          progressiveRefresh: refresh,
        ),
      ),
    ),
  );
}

const _firstUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap='
    '$testPlaybackCapability';
const _staleUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap='
    'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
const _latestCapability = 'ccccccccccccccccccccccccccccccccccccccccccc';
const _latestUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap=$_latestCapability';
