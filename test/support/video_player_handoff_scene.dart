import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

typedef VideoPlayerHandoffScene = ({String active, VideoPlaybackMode mode});

Future<void> pumpVideoPlayerHandoffScene(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  VideoPlayerHandoffScene scene,
) async {
  await tester.pumpWidget(
    MaterialApp(
      home: Stack(
        children: ['a', 'b'].map((id) {
          return port.buildSurface(
            VideoPlaybackSurfaceRequest(
              media: VideoMediaSource.local('/cache/$id.mp4'),
              isActive: id == 'a' || id == scene.active,
              mode: id == scene.active ? scene.mode : VideoPlaybackMode.normal,
            ),
          );
        }).toList(),
      ),
    ),
  );
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}
