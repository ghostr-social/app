import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';

void main() {
  testWidgets('uses an explicitly trusted HLS network data source', (
    tester,
  ) async {
    final platform = FakeVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final media = ProxiedHlsVideoMediaSource(
      'http://127.0.0.1:3210/hls/'
      '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef/'
      'index.m3u8',
    );

    await tester.pumpWidget(
      MaterialApp(
        home: VideoPlayerPlaybackPort().buildSurface(
          VideoPlaybackSurfaceRequest(media: media, isActive: true),
        ),
      ),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(platform.dataSources.single.sourceType, DataSourceType.network);
    expect(platform.dataSources.single.formatHint, VideoFormat.hls);
    expect(platform.dataSources.single.uri, media.playbackUri.toString());
  });
}
