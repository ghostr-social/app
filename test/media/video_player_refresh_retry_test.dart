import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player/video_player.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/recovering_video_player_platform.dart';
import '../support/scripted_progressive_playback_refresh.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('retries a transient capability renewal failure', (tester) async {
    final platform = RecoveringVideoPlayerPlatform();
    final refresh = ScriptedProgressivePlaybackRefresh();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort(
      recoveryPolicy: PlaybackRecoveryPolicy([Duration.zero, Duration.zero]),
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
    await _failFirstRenewal(tester, platform, refresh);
    expect(refresh.requestCount, 2);
    refresh.completeNext(_renewedUrl);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    await settleVideoPlayerTasks(tester);

    expect(platform.dataSources, hasLength(2));
    expect(platform.dataSources.last.uri, _renewedUrl);
    expect(find.byType(VideoPlayer), findsOneWidget);
    expect(find.text('Video unavailable'), findsNothing);
  });
}

Future<void> _failFirstRenewal(
  WidgetTester tester,
  RecoveringVideoPlayerPlatform platform,
  ScriptedProgressivePlaybackRefresh refresh,
) async {
  platform.failLatest('source interrupted');
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
  refresh.failNext();
  await tester.pump();
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}

const _firstUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap='
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const _renewedUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap='
    'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
