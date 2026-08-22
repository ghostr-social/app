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
  testWidgets('replacement waits for retirement and observes its failure', (
    tester,
  ) async {
    final platform = FeedPreparationVideoPlayerPlatform()..blockDisposal();
    VideoPlayerPlatform.instance = platform;
    addTearDown(platform.releaseDisposal);
    final port = VideoPlayerPlaybackPort(
      recoveryPolicy: PlaybackRecoveryPolicy([Duration.zero]),
    );
    await _show(tester, port);

    platform.fail(0);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    await settleVideoPlayerTasks(tester);
    expect(platform.createdCount, 1);

    platform.releaseDisposal();
    await settleVideoPlayerTasks(tester);
    expect(platform.createdCount, 2);
    await tester.pump(const Duration(milliseconds: 1));
    platform.fail(1);
    await settleVideoPlayerTasks(tester);

    expect(find.bySemanticsLabel('Video unavailable'), findsOneWidget);
    expect(platform.isPlaying(1), isFalse);
    expect(platform.audibleOverlap, isFalse);
  });
}

Future<void> _show(WidgetTester tester, VideoPlayerPlaybackPort port) async {
  await tester.pumpWidget(
    MaterialApp(
      home: port.buildSurface(
        VideoPlaybackSurfaceRequest(
          media: VideoMediaSource.local('/cache/clip.mp4'),
          isActive: true,
        ),
      ),
    ),
  );
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}
