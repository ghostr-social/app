import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/feed_preparation_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('retirement during mute starts bounded teardown proof', (
    tester,
  ) async {
    final platform = FeedPreparationVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final stalled = Completer<void>();
    var disposals = 0;
    final port = VideoPlayerPlaybackPort(
      recoveryPolicy: const PlaybackRecoveryPolicy.disabled(),
      controllerDisposer: (controller) async {
        if (disposals++ == 0) {
          await controller.pause();
          await stalled.future;
          return;
        }
        await controller.dispose();
      },
    );
    await _show(tester, port, (activeB: false, keepA: true));
    platform.blockNextMute();
    addTearDown(platform.releaseMute);
    await _show(tester, port, (activeB: true, keepA: true));
    await platform.muteEntered;

    await _show(tester, port, (activeB: true, keepA: false));
    await tester.pump(playbackControllerTeardownTimeout);
    await tester.pump(const Duration(milliseconds: 1));

    expect(find.bySemanticsLabel('Video unavailable'), findsOneWidget);
    expect(platform.audibleOverlap, isFalse);
    platform.releaseMute();
    await tester.pumpWidget(const SizedBox());
    await settleVideoPlayerTasks(tester);
  });
}

Future<void> _show(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  ({bool activeB, bool keepA}) scene,
) async {
  await tester.pumpWidget(
    MaterialApp(
      home: Stack(
        children: ['a', 'b'].map((id) {
          return port.buildSurface(
            VideoPlaybackSurfaceRequest(
              media: VideoMediaSource.local('/cache/$id.mp4'),
              isActive: id == 'a' ? scene.keepA : scene.activeB,
              mode: VideoPlaybackMode.normal,
            ),
          );
        }).toList(),
      ),
    ),
  );
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}
