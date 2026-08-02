import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:video_player/video_player.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';

void main() {
  testWidgets('plays a fallback URL when primary initialization fails',
      (tester) async {
    final platform = FakeVideoPlayerPlatform()..failNextInitialization = true;
    VideoPlayerPlatform.instance = platform;
    final media = VideoMediaSource.remote(
      'https://media.example/primary.mp4',
      fallbackUrls: ['https://media.example/fallback.mp4'],
    );

    await tester.pumpWidget(MaterialApp(
      home: const VideoPlayerPlaybackPort().buildSurface(
        media: media,
        isActive: true,
      ),
    ));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(find.byType(VideoPlayer), findsOneWidget);
    expect(platform.dataSources.map((source) => source.uri), [
      'https://media.example/primary.mp4',
      'https://media.example/fallback.mp4',
    ]);
  });
}
