import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/feed_preparation_video_player_platform.dart';

void main() {
  testWidgets('a second cover cancels queued replacement loading', (
    tester,
  ) async {
    final platform = FeedPreparationVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort();
    final media = VideoMediaSource.local('/cache/video.mp4');
    platform.blockNextMute();
    addTearDown(platform.releaseMute);

    await _show(tester, port, media, true);
    await tester.runAsync(
      () => platform.muteEntered.timeout(const Duration(seconds: 1)),
    );
    platform.blockDisposal();
    addTearDown(platform.releaseDisposal);
    expect(platform.createdCount, 1);
    await _show(tester, port, media, false);
    await _show(tester, port, media, true);
    await _show(tester, port, media, false);
    platform.releaseDisposal();
    await _drain(tester);

    expect(platform.createdCount, 1);
    expect(platform.playerCount, 0);
  });
}

Future<void> _show(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  VideoMediaSource media,
  bool active,
) async {
  await tester.pumpWidget(
    MaterialApp(
      home: port.buildSurface(
        VideoPlaybackSurfaceRequest(
          media: media,
          isActive: active,
          mode: VideoPlaybackMode.paused,
        ),
      ),
    ),
  );
  await tester.pump(const Duration(milliseconds: 1));
}

Future<void> _drain(WidgetTester tester) async {
  for (var turn = 0; turn < 8; turn += 1) {
    await tester.runAsync(() => Future<void>.delayed(Duration.zero));
    await tester.pump(const Duration(milliseconds: 1));
  }
}
