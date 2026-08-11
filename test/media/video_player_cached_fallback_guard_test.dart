import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';

void main() {
  testWidgets('does not fall back from a local file to an unverified URL', (
    tester,
  ) async {
    final platform = FakeVideoPlayerPlatform()..failNextInitialization = true;
    VideoPlayerPlatform.instance = platform;
    final media = VideoMediaSource.cached(
      '/missing/video.mp4',
      remoteUrl: 'http://127.0.0.1/fallback.mp4',
    );

    await tester.pumpWidget(
      MaterialApp(
        home:
            VideoPlayerPlaybackPort(
              recoveryPolicy: PlaybackRecoveryPolicy.disabled(),
            ).buildSurface(
              VideoPlaybackSurfaceRequest(media: media, isActive: true),
            ),
      ),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(find.text('Video unavailable'), findsOneWidget);
    expect(platform.dataSources, hasLength(1));
    expect(platform.dataSources.single.sourceType, DataSourceType.file);
  });
}
