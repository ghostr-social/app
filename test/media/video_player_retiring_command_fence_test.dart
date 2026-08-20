import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player/video_player.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/late_video_player_command_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('retirement waits for an outstanding native play command', (
    tester,
  ) async {
    final result = await _runScenario(tester);
    expect(result.disposalRacedPlay, isFalse);
    expect(result.livePlayers, 0);
  });
}

Future<({bool disposalRacedPlay, int livePlayers})> _runScenario(
  WidgetTester tester,
) async {
  final platform = LateVideoPlayerCommandPlatform()
    ..blockNext(LateVideoPlayerCommand.play);
  VideoPlayerPlatform.instance = platform;
  final entered = Completer<void>();
  final allowed = Completer<void>();
  final port = VideoPlayerPlaybackPort(
    controllerDisposer: (controller) =>
        _disposeAfter(controller, entered, allowed.future),
  );
  await _show(tester, port);
  await platform.entered;
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
  final raced = entered.isCompleted;
  platform.release();
  await entered.future;
  allowed.complete();
  await settleVideoPlayerTasks(tester);
  return (disposalRacedPlay: raced, livePlayers: platform.playerCount);
}

Future<void> _disposeAfter(
  VideoPlayerController controller,
  Completer<void> entered,
  Future<void> allowed,
) async {
  entered.complete();
  await allowed;
  await controller.dispose();
}

Future<void> _show(WidgetTester tester, VideoPlayerPlaybackPort port) async {
  await tester.pumpWidget(
    MaterialApp(
      home: port.buildSurface(
        VideoPlaybackSurfaceRequest(
          media: VideoMediaSource.local('/cache/a.mp4'),
          isActive: true,
        ),
      ),
    ),
  );
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}
