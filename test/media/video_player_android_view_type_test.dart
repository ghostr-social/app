import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/capturing_video_view_platform.dart';

void main() {
  testWidgets('uses the Android texture view for feed video', (tester) async {
    debugDefaultTargetPlatformOverride = TargetPlatform.android;
    try {
      final platform = CapturingVideoViewPlatform();
      VideoPlayerPlatform.instance = platform;

      await tester.pumpWidget(
        MaterialApp(
          home: const VideoPlayerPlaybackPort().buildSurface(
            media: ProxiedProgressiveVideoMediaSource(
              'http://127.0.0.1:3210/video.mp4?id=post-1',
            ),
            isActive: true,
          ),
        ),
      );
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 100));

      expect(
        platform.creationOptions.single.viewType,
        VideoViewType.textureView,
      );
    } finally {
      debugDefaultTargetPlatformOverride = null;
    }
  });
}
