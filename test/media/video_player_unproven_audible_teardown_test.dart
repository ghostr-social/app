import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player/video_player.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/audited_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('an unproven audible teardown fails the next player closed', (
    tester,
  ) async {
    final platform = AuditedVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    var disposalAttempts = 0;
    var released = 0;
    VideoPlayerController? failedController;
    final port = VideoPlayerPlaybackPort(
      controllerDisposer: (controller) async {
        if (++disposalAttempts == 1) {
          failedController = controller;
          throw StateError('injected teardown failure');
        }
        await controller.dispose();
      },
      recoveryPolicy: PlaybackRecoveryPolicy.disabled(),
    );

    await pumpVideoPlayerSurface(
      tester,
      port,
      _request('a', () => released += 1),
    );
    expect(platform.isPlaying(0), isTrue);
    await pumpVideoPlayerSurface(tester, port, _request('b'));
    await settleVideoPlayerTasks(tester);

    expect(find.bySemanticsLabel('Video unavailable'), findsOneWidget);
    expect(platform.playerCount, 1);
    expect(platform.isPlaying(0), isTrue);
    expect(platform.audibleOverlap, isFalse);
    expect(released, 0);
    await failedController!.pause();
  });
}

VideoPlaybackSurfaceRequest _request(String id, [VoidCallback? released]) {
  return VideoPlaybackSurfaceRequest(
    media: VideoMediaSource.local('/cache/$id.mp4'),
    isActive: true,
    onPlaybackMediaReleased: released,
  );
}
