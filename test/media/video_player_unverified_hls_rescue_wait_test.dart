import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('unverified HLS failure waits for transport rescue', (
    tester,
  ) async {
    final platform = FakeVideoPlayerPlatform()..failNextInitialization = true;
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort(
      recoveryPolicy: const PlaybackRecoveryPolicy.disabled(),
    );

    await pumpVideoPlayerSurface(
      tester,
      port,
      VideoPlaybackSurfaceRequest(
        media: ProxiedHlsVideoMediaSource(_playbackUrl),
        playbackDeliveryId: PlaybackDeliveryId.parse('policy-hls'),
        isActive: true,
      ),
    );
    await settleVideoPlayerTasks(tester);

    expect(find.bySemanticsLabel('Loading video'), findsOneWidget);
    expect(find.text('Video unavailable'), findsNothing);
    expect(platform.dataSources, hasLength(1));

    await tester.pump(playbackCapabilityRescueTimeout);

    expect(find.text('Video unavailable'), findsOneWidget);
  });
}

const _playbackUrl =
    'http://127.0.0.1:3210/hls/'
    '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef/'
    'index.m3u8';
