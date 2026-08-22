import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/feed_preparation_video_player_platform.dart';

void main() {
  testWidgets('replacement waits until the audible player is disposed', (
    tester,
  ) async {
    final platform = FeedPreparationVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort();
    platform.blockDisposal();
    addTearDown(platform.releaseDisposal);

    await _show(tester, port, const ['a'], 'a');
    await _show(tester, port, const ['a', 'b'], 'b');
    expect(platform.isPlaying(1), isFalse);
    expect(platform.audibleOverlap, isFalse);

    platform.releaseDisposal();
    await _drain(tester);
    expect(platform.isPlaying(1), isTrue);
    expect(platform.audibleOverlap, isFalse);
  });
}

Future<void> _show(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  List<String> ids,
  String active,
) async {
  await tester.pumpWidget(
    MaterialApp(
      home: Stack(
        children: ids.map((id) {
          return port.buildSurface(
            VideoPlaybackSurfaceRequest(
              media: VideoMediaSource.local('/cache/$id.mp4'),
              isActive: id == active,
            ),
          );
        }).toList(),
      ),
    ),
  );
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}

Future<void> _drain(WidgetTester tester) async {
  await tester.runAsync(() => Future<void>.delayed(Duration.zero));
  await tester.pump(const Duration(milliseconds: 100));
}
