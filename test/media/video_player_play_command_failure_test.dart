import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';

void main() {
  testWidgets('shows a retryable state when play fails', (tester) async {
    VideoPlayerPlatform.instance = FakeVideoPlayerPlatform()
      ..failingCalls.add('play');

    await tester.pumpWidget(
      MaterialApp(
        home:
            VideoPlayerPlaybackPort(
              recoveryPolicy: PlaybackRecoveryPolicy.disabled(),
            ).buildSurface(
              VideoPlaybackSurfaceRequest(
                media: VideoMediaSource.local('/cache/video.mp4'),
                isActive: true,
              ),
            ),
      ),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(find.text('Video unavailable'), findsOneWidget);
  });
}
