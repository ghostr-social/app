import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/audited_video_player_platform.dart';

void main() {
  testWidgets('a late valid initialization remains the owned controller', (
    tester,
  ) async {
    final platform = AuditedVideoPlayerPlatform(autoInitialize: false);
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort(
      recoveryPolicy: PlaybackRecoveryPolicy([Duration.zero]),
    );
    final request = VideoPlaybackSurfaceRequest(
      media: VideoMediaSource.local('/cache/video.mp4'),
      isActive: true,
    );

    await tester.pumpWidget(MaterialApp(home: port.buildSurface(request)));
    await tester.pump();
    expect(platform.createdCount, 1);
    await tester.pump(const Duration(seconds: 3));
    await tester.runAsync(() => Future<void>.delayed(Duration.zero));
    await tester.pump();

    expect(platform.createdCount, 1);
    expect(platform.playerCount, 1);
    platform.initialize(0);
    await tester.pump();
    expect(find.byType(Texture), findsOneWidget);
    expect(platform.commands, contains('play:0'));
    expect(find.bySemanticsLabel('Video unavailable'), findsNothing);
  });
}
