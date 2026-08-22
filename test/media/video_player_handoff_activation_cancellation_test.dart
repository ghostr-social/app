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
  testWidgets('superseded handoff rearms teardown proof for current intent', (
    tester,
  ) async {
    final platform = FeedPreparationVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    var disposals = 0;
    final stalledDisposal = Completer<void>();
    final port = VideoPlayerPlaybackPort(
      recoveryPolicy: const PlaybackRecoveryPolicy.disabled(),
      controllerDisposer: (controller) async {
        if (disposals++ == 0) {
          await controller.pause();
          await stalledDisposal.future;
          return;
        }
        await controller.dispose();
      },
    );

    await _show(tester, port, (
      ids: ['a'],
      active: 'a',
      mode: VideoPlaybackMode.normal,
    ));
    await _show(tester, port, (
      ids: ['a', 'b'],
      active: 'b',
      mode: VideoPlaybackMode.normal,
    ));
    await _show(tester, port, (
      ids: ['a', 'b'],
      active: 'b',
      mode: VideoPlaybackMode.paused,
    ));
    await _show(tester, port, (
      ids: ['a', 'b'],
      active: 'b',
      mode: VideoPlaybackMode.normal,
    ));
    await tester.pump(playbackControllerTeardownTimeout);
    await tester.pump(const Duration(milliseconds: 1));

    expect(find.bySemanticsLabel('Video unavailable'), findsOneWidget);
    expect(platform.audibleOverlap, isFalse);
    await tester.pumpWidget(const SizedBox());
    await settleVideoPlayerTasks(tester);
  });
}

Future<void> _show(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  ({List<String> ids, String active, VideoPlaybackMode mode}) scene,
) async {
  await tester.pumpWidget(
    MaterialApp(
      home: Stack(
        children: scene.ids.map((id) {
          return port.buildSurface(
            VideoPlaybackSurfaceRequest(
              media: VideoMediaSource.local('/cache/$id.mp4'),
              isActive: id == scene.active,
              mode: id == scene.active ? scene.mode : VideoPlaybackMode.paused,
            ),
          );
        }).toList(),
      ),
    ),
  );
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}
