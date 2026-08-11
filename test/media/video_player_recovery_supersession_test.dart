import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/recovering_video_player_platform.dart';

void main() {
  testWidgets('a superseded surface cancels its delayed reconstruction', (
    tester,
  ) async {
    final platform = RecoveringVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort(
      recoveryPolicy: PlaybackRecoveryPolicy([
        const Duration(milliseconds: 500),
      ]),
    );

    await pump(tester, port, '/cache/first.mp4');
    platform.failLatest('first source failed');
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    await pump(tester, port, '/cache/second.mp4');
    await tester.pump(const Duration(seconds: 1));

    expect(platform.dataSources, hasLength(2));
    expect(platform.dataSources.last.uri, contains('second.mp4'));
  });
}

Future<void> pump(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  String path,
) async {
  await tester.pumpWidget(
    MaterialApp(
      home: port.buildSurface(
        VideoPlaybackSurfaceRequest(
          media: VideoMediaSource.local(path),
          isActive: true,
        ),
      ),
    ),
  );
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}
