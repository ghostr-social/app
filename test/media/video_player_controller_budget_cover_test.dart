import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/feed_preparation_video_player_platform.dart';

void main() {
  testWidgets('covering a queued active surface cancels its decoder claim', (
    tester,
  ) async {
    final platform = FeedPreparationVideoPlayerPlatform(autoInitialize: false);
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort();
    final media = {
      for (final id in ['a', 'b', 'c'])
        id: VideoMediaSource.local('/cache/$id.mp4'),
    };

    await _show(tester, port, media, (ids: ['a', 'b'], active: 'b'));
    platform.blockDisposal();
    addTearDown(platform.releaseDisposal);
    await _show(tester, port, media, (ids: ['a', 'b', 'c'], active: 'c'));
    await _show(tester, port, media, (ids: ['a', 'b', 'c'], active: 'a'));
    platform.releaseDisposal();
    await _drain(tester);

    expect(platform.createdCount, 2);
    expect(platform.playerCount, 1);
  });
}

Future<void> _show(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  Map<String, VideoMediaSource> media,
  ({List<String> ids, String active}) scene,
) async {
  await tester.pumpWidget(
    MaterialApp(
      home: Stack(
        children: scene.ids.map((id) {
          return port.buildSurface(
            VideoPlaybackSurfaceRequest(
              media: media[id]!,
              isActive: id == scene.active,
              mode: VideoPlaybackMode.paused,
            ),
          );
        }).toList(),
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
