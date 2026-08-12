import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';

void main() {
  testWidgets('rejects remote sources before creating a platform player', (
    tester,
  ) async {
    final platform = FakeVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final media = VideoMediaSource.remote(
      'http://127.0.0.1/live.m3u8',
      delivery: VideoMediaDelivery.hls,
    );

    await tester.pumpWidget(
      MaterialApp(
        home: VideoPlayerPlaybackPort().buildSurface(
          VideoPlaybackSurfaceRequest(media: media, isActive: true),
        ),
      ),
    );
    await tester.pump();

    expect(find.text('Video unavailable'), findsOneWidget);
    expect(platform.dataSources, isEmpty);
  });
}
