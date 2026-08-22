import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/feed_preparation_video_player_platform.dart';

void main() {
  testWidgets('reactivating a queued surface creates one controller', (
    tester,
  ) async {
    final platform = FeedPreparationVideoPlayerPlatform(autoInitialize: false);
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort();
    final media = {
      for (final id in ['a', 'b', 'c', 'd'])
        id: VideoMediaSource.local('/cache/$id.mp4'),
    };

    await _show(tester, port, media, (ids: ['a', 'b'], active: 'b'));
    expect(platform.playerCount, 2);
    await _show(tester, port, media, (ids: ['a', 'b', 'c'], active: 'b'));
    expect(platform.createdCount, 2);
    platform.blockDisposal();
    addTearDown(platform.releaseDisposal);
    await _show(tester, port, media, (ids: ['a', 'b', 'c'], active: 'c'));
    expect(platform.createdCount, 2);
    await _show(tester, port, media, (ids: ['c'], active: 'c'));
    platform.releaseDisposal();
    await _drain(tester);
    platform.initialize(2);
    await _drain(tester);
    await _show(tester, port, media, (ids: ['c', 'd'], active: 'c'));

    expect(_creations(platform, 'c'), 1);
    expect(_creations(platform, 'd'), 1);
    expect(platform.playerCount, 2);
  });
}

int _creations(FeedPreparationVideoPlayerPlatform platform, String id) {
  return platform.sources.values
      .where((source) => source.uri?.endsWith('/$id.mp4') == true)
      .length;
}

Future<void> _turn(WidgetTester tester) async {
  await tester.runAsync(() async {
    for (var turn = 0; turn < 4; turn += 1) {
      await Future<void>.delayed(Duration.zero);
    }
  });
  await tester.pump();
}

Future<void> _drain(WidgetTester tester) async {
  for (var turn = 0; turn < 12; turn += 1) {
    await _turn(tester);
  }
  await tester.pump();
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
  await tester.pump(const Duration(milliseconds: 100));
  await _turn(tester);
}
