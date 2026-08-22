import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player/video_player.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('rapid retry taps start one replacement player', (tester) async {
    final platform = FakeVideoPlayerPlatform()..failNextInitialization = true;
    VideoPlayerPlatform.instance = platform;

    await tester.pumpWidget(
      MaterialApp(
        home:
            VideoPlayerPlaybackPort(
              recoveryPolicy: PlaybackRecoveryPolicy.disabled(),
            ).buildSurface(
              VideoPlaybackSurfaceRequest(
                media: VideoMediaSource.local('/cache/unavailable.mp4'),
                isActive: true,
              ),
            ),
      ),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    expect(find.text('Retry'), findsOneWidget);

    await tester.tap(find.text('Retry'));
    await tester.tap(find.text('Retry'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    await settleVideoPlayerTasks(tester);

    expect(find.byType(VideoPlayer), findsOneWidget);
    expect(platform.dataSources, hasLength(2));
  });
}
