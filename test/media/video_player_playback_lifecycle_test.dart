import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player/video_player.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';

void main() {
  testWidgets('plays the active source and resets replacement controllers', (
    tester,
  ) async {
    final platform = FakeVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    const port = VideoPlayerPlaybackPort();
    final first = VideoMediaSource.local('/cache/first.mp4');

    await tester.pumpWidget(
      MaterialApp(
        home: port.buildSurface(
          VideoPlaybackSurfaceRequest(media: first, isActive: true),
        ),
      ),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(find.byType(VideoPlayer), findsOneWidget);
    expect(
      platform.dataSources.single.uri,
      Uri.file(first.localPath!).toString(),
    );
    expect(platform.calls, containsAllInOrder(['setLooping', 'play']));

    await tester.pumpWidget(
      MaterialApp(
        home: port.buildSurface(
          VideoPlaybackSurfaceRequest(media: first, isActive: false),
        ),
      ),
    );
    await tester.pump();
    expect(platform.calls, containsAllInOrder(['pause', 'seekTo']));

    final local = VideoMediaSource.local('/cache/second.mp4');
    await tester.pumpWidget(
      MaterialApp(
        home: port.buildSurface(
          VideoPlaybackSurfaceRequest(media: local, isActive: false),
        ),
      ),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    await tester.pump();
    expect(platform.dataSources, hasLength(2));
    expect(
      platform.dataSources.last.uri,
      Uri.file(local.localPath!).toString(),
    );
  });
}
