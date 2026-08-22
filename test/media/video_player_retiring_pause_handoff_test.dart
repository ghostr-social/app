import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/feed_preparation_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('retiring a muted player releases its blocked pause handoff', (
    tester,
  ) async {
    final platform = FeedPreparationVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final pause = platform.pauseGate = Completer<void>();
    final stalled = Completer<void>();
    var disposals = 0;
    final port = VideoPlayerPlaybackPort(
      controllerDisposer: (controller) async {
        if (disposals++ == 0) {
          await controller.pause();
          await stalled.future;
          return;
        }
        await controller.dispose();
      },
    );
    addTearDown(() {
      if (!pause.isCompleted) pause.complete();
    });

    await _show(tester, port, 'a', VideoPlaybackMode.normal);
    await _show(tester, port, 'a', VideoPlaybackMode.paused);
    await _show(tester, port, 'b', VideoPlaybackMode.normal);
    await settleVideoPlayerTasks(tester);

    expect(platform.isPlaying(platform.playerFor('/cache/b.mp4')), isTrue);
    expect(platform.audibleOverlap, isFalse);
    pause.complete();
    await tester.pumpWidget(const SizedBox());
    await settleVideoPlayerTasks(tester);
  });
}

Future<void> _show(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  String id,
  VideoPlaybackMode mode,
) async {
  await tester.pumpWidget(
    MaterialApp(
      home: port.buildSurface(
        VideoPlaybackSurfaceRequest(
          media: VideoMediaSource.local('/cache/$id.mp4'),
          isActive: true,
          mode: mode,
        ),
      ),
    ),
  );
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}
