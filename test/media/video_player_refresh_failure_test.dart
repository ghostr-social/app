import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/recovering_video_player_platform.dart';
import '../support/scripted_progressive_playback_refresh.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('failed capability renewal exhausts into accessible retry', (
    tester,
  ) async {
    final platform = RecoveringVideoPlayerPlatform();
    final refresh = ScriptedProgressivePlaybackRefresh();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort(
      recoveryPolicy: PlaybackRecoveryPolicy([Duration.zero]),
    );
    await pumpVideoPlayerSurface(
      tester,
      port,
      VideoPlaybackSurfaceRequest(
        media: ProxiedProgressiveVideoMediaSource(_firstUrl),
        isActive: true,
        progressiveRefresh: refresh,
      ),
    );
    await _failCapabilityRenewal(tester, platform, refresh);
    expect(find.bySemanticsLabel('Video unavailable'), findsOneWidget);
    expect(find.bySemanticsLabel('Retry'), findsOneWidget);
    expect(platform.dataSources, hasLength(1));
    await _retryCapabilityRenewal(tester, refresh);
    expect(platform.dataSources, hasLength(2));
    expect(platform.dataSources.last.uri, _secondUrl);
  });
}

Future<void> _failCapabilityRenewal(
  WidgetTester tester,
  RecoveringVideoPlayerPlatform platform,
  ScriptedProgressivePlaybackRefresh refresh,
) async {
  platform.failLatest('source interrupted');
  await tester.pump();
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
  expect(refresh.requestCount, 1);
  refresh.failNext();
  await tester.pump();
  await tester.pump();
}

Future<void> _retryCapabilityRenewal(
  WidgetTester tester,
  ScriptedProgressivePlaybackRefresh refresh,
) async {
  await tester.tap(find.text('Retry'));
  await tester.pump();
  await tester.pump();
  expect(refresh.requestCount, 2);
  refresh.completeNext(_secondUrl);
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
  await settleVideoPlayerTasks(tester);
}

const _firstUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap='
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const _secondUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap='
    'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
