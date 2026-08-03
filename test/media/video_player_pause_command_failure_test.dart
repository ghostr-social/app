import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';

void main() {
  testWidgets('shows a retryable state when pause fails', (tester) async {
    final platform = FakeVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    const port = VideoPlayerPlaybackPort();
    final media = VideoMediaSource.local('/cache/video.mp4');

    await tester.pumpWidget(MaterialApp(
      home: port.buildSurface(media: media, isActive: true),
    ));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    platform.failingCalls.add('pause');

    await tester.pumpWidget(MaterialApp(
      home: port.buildSurface(media: media, isActive: false),
    ));
    await tester.pump();

    expect(find.text('Video unavailable'), findsOneWidget);
  });
}
