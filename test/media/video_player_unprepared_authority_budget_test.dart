import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/feed_preparation_video_player_platform.dart';
import '../support/playback_authority_fixture.dart';

void main() {
  testWidgets('asset authority alone cannot unlock prepared decoder reserve', (
    tester,
  ) async {
    final platform = FeedPreparationVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort();

    await tester.pumpWidget(
      MaterialApp(
        home: Stack(
          children: ['p0', 'p1', 'p2'].map((id) => _surface(port, id)).toList(),
        ),
      ),
    );
    await tester.pump(const Duration(milliseconds: 100));

    expect(platform.createdCount, 2);
  });
}

Widget _surface(VideoPlayerPlaybackPort port, String id) {
  final media = ProxiedProgressiveVideoMediaSource(
    'http://127.0.0.1:3210/video.mp4?id=$id&cap=$testPlaybackCapability',
  );
  return port.buildSurface(
    VideoPlaybackSurfaceRequest(
      media: media,
      isActive: id == 'p0',
      authority: testPlaybackAuthority(postId: id),
    ),
  );
}
