import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/recovering_video_player_platform.dart';

void main() {
  testWidgets('defers failed inactive playback until the surface is active', (
    tester,
  ) async {
    final platform = RecoveringVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort(
      recoveryPolicy: PlaybackRecoveryPolicy([Duration.zero]),
    );
    final media = VideoMediaSource.local('/cache/video.mp4');

    await pumpSurface(tester, port, media, isActive: false);
    platform.failLatest('inactive decoder failed');
    await tester.pump();
    await tester.pump(const Duration(seconds: 1));

    expect(platform.dataSources, hasLength(1));
    expect(find.text('Video unavailable'), findsNothing);

    await pumpSurface(tester, port, media, isActive: true);

    expect(platform.dataSources, hasLength(2));
    expect(platform.commands.last, 'play:1');
  });
}

Future<void> pumpSurface(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  VideoMediaSource media, {
  required bool isActive,
}) async {
  await tester.pumpWidget(
    MaterialApp(
      home: port.buildSurface(
        VideoPlaybackSurfaceRequest(media: media, isActive: isActive),
      ),
    ),
  );
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}
