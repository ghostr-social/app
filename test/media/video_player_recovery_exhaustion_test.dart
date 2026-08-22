import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/recovering_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('exhaustion is accessible and manual retry starts a new budget', (
    tester,
  ) async {
    final platform = RecoveringVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort(
      recoveryPolicy: PlaybackRecoveryPolicy([Duration.zero]),
    );
    await pumpVideoPlayerSurface(
      tester,
      port,
      VideoPlaybackSurfaceRequest(
        media: VideoMediaSource.local('/cache/video.mp4'),
        isActive: true,
      ),
    );
    platform.position = const Duration(seconds: 4);
    await tester.pump(const Duration(milliseconds: 100));
    await _exhaustRecovery(tester, platform);
    expect(find.bySemanticsLabel('Video unavailable'), findsOneWidget);
    expect(find.bySemanticsLabel('Retry'), findsOneWidget);
    expect(platform.dataSources, hasLength(2));
    await _retryManually(tester);
    expect(platform.dataSources, hasLength(3), reason: '${platform.commands}');
    expect(platform.commands, containsAllInOrder(['seek:2:4000', 'play:2']));
  });
}

Future<void> _exhaustRecovery(
  WidgetTester tester,
  RecoveringVideoPlayerPlatform platform,
) async {
  platform.failLatest('first interruption');
  await tester.pump();
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
  await settleVideoPlayerTasks(tester);
  platform.failLatest('second interruption');
  await tester.pump();
  await tester.pump();
  await tester.pump();
  await settleVideoPlayerTasks(tester);
}

Future<void> _retryManually(WidgetTester tester) async {
  await tester.tap(find.text('Retry'));
  await tester.pump();
  await settleVideoPlayerTasks(tester);
  await tester.pump(const Duration(milliseconds: 100));
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}
