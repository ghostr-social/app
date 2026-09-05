import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/rendered_first_frame_protocol.dart';
import 'package:video_player/video_player.dart';

import 'live_video_log.dart';
import 'live_comparison_surface.dart';

// The same installed native player, with only the WARP gateway bypassed.
Future<void> liveDirectPlayback(
  WidgetTester tester,
  LiveVideoLog log,
  Uri url,
) async {
  final clock = Stopwatch()..start();
  final attempt = NativeRenderedFirstFramePort.production().beginAttempt();
  Duration? rendered;
  attempt?.listen(() => rendered = clock.elapsed);
  final controller = VideoPlayerController.networkUrl(
    url,
    httpHeaders: {
      if (attempt != null) warpPlaybackAttemptHeader: attempt.token.value,
    },
  );
  try {
    await _play(tester, controller);
    final initialized = clock.elapsedMilliseconds;
    await _observe(tester, controller);
    log.add('direct_player', {
      'url': '$url',
      'initializedMs': initialized,
      'firstFrameMs': rendered?.inMilliseconds,
      'positionMs': controller.value.position.inMilliseconds,
      'buffering': controller.value.isBuffering,
      'error': controller.value.errorDescription,
    });
  } on Object catch (error) {
    log.add('direct_player', {
      'url': '$url',
      'durationMs': clock.elapsedMilliseconds,
      'error': '$error',
    });
  } finally {
    await tester.pumpWidget(const SizedBox.shrink());
    await controller.dispose();
    attempt?.release();
  }
}

Future<void> _play(
  WidgetTester tester,
  VideoPlayerController controller,
) async {
  await controller.initialize().timeout(const Duration(seconds: 30));
  await controller.setVolume(0);
  await tester.pumpWidget(
    LiveComparisonSurface(
      label: 'Direct playback',
      host: Uri.parse(controller.dataSource).host,
      child: AspectRatio(
        aspectRatio: controller.value.aspectRatio,
        child: VideoPlayer(controller),
      ),
    ),
  );
  await controller.play();
}

Future<void> _observe(WidgetTester tester, VideoPlayerController player) async {
  final clock = Stopwatch()..start();
  while (clock.elapsed < const Duration(seconds: 10)) {
    await tester.pump(const Duration(milliseconds: 50));
    if (player.value.hasError) return;
  }
}
