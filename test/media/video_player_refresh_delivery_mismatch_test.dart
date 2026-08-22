import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/playback_authority_fixture.dart';
import '../support/recovering_video_player_platform.dart';
import '../support/scripted_progressive_playback_refresh.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('refresh cannot replace playback with another delivery', (
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
        authority: testPlaybackAuthority(),
        progressiveRefresh: refresh,
      ),
    );

    platform.failLatest('source interrupted');
    await tester.pump(const Duration(milliseconds: 100));
    refresh.completeNext(_otherDeliveryUrl);
    await tester.pump(const Duration(milliseconds: 100));
    await settleVideoPlayerTasks(tester);

    expect(platform.dataSources, hasLength(1));
    expect(find.bySemanticsLabel('Video unavailable'), findsOneWidget);
  });
}

const _firstUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap='
    '$testPlaybackCapability';
const _otherDeliveryUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-2&cap='
    'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
