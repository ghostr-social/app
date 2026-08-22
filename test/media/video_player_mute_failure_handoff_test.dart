import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/feed_preparation_video_player_platform.dart';

void main() {
  testWidgets('failed mute retains the audible owner for a safe retry', (
    tester,
  ) async {
    final platform = FeedPreparationVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort();
    final media = {
      for (final id in ['a', 'b', 'c'])
        id: VideoMediaSource.local('/cache/$id.mp4'),
    };

    await _show(tester, port, media, ids: ['a'], active: {'a'});
    platform.failNextMute(platform.sources.keys.single);
    await _show(tester, port, media, ids: ['a', 'b'], active: {'a', 'b'});
    await _drain(tester);
    expect(platform.muteFailures, 1);
    expect(platform.isPlaying(platform.playerFor('/cache/a.mp4')), isTrue);

    await _show(tester, port, media, ids: ['a', 'c'], active: {'a', 'c'});
    await _drain(tester);
    expect(platform.audibleOverlap, isFalse, reason: '${platform.commands}');
    expect(
      platform.isPlaying(platform.playerFor('/cache/c.mp4')),
      isTrue,
      reason: '${platform.commands}',
    );
    expect(
      platform.commands.where((command) => command == 'volume:0:0.0').length,
      greaterThanOrEqualTo(2),
    );
  });
}

Future<void> _show(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  Map<String, VideoMediaSource> media, {
  required List<String> ids,
  required Set<String> active,
}) async {
  await tester.pumpWidget(
    MaterialApp(
      home: Stack(
        children: ids
            .map(
              (id) => port.buildSurface(
                VideoPlaybackSurfaceRequest(
                  media: media[id]!,
                  isActive: active.contains(id),
                ),
              ),
            )
            .toList(),
      ),
    ),
  );
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
  await tester.runAsync(() => Future<void>.delayed(Duration.zero));
  await tester.pump();
}

Future<void> _drain(WidgetTester tester) async {
  for (var turn = 0; turn < 4; turn += 1) {
    await tester.runAsync(() => Future<void>.delayed(Duration.zero));
    await tester.pump(const Duration(milliseconds: 1));
  }
}
