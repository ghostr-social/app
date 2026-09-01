import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/hls_video_playback_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/feed_preparation_video_player_platform.dart';
import '../support/fake_hls_playback_gateway.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('capacity stays occupied until controller teardown settles', (
    tester,
  ) async {
    final platform = FeedPreparationVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final port = HlsVideoPlaybackPort(
      delegate: VideoPlayerPlaybackPort(),
      gateway: FakeHlsPlaybackGateway(),
    );
    await tester.pumpWidget(
      MaterialApp(
        home: port.buildSurface(
          VideoPlaybackSurfaceRequest(
            media: VideoMediaSource.local('/cache/capacity.mp4'),
            isActive: false,
          ),
        ),
      ),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(port.capacitySnapshot.inUse, 1);
    expect(port.capacitySnapshot.isQuiescent, isFalse);
    platform.blockDisposal();
    await tester.pumpWidget(const MaterialApp(home: SizedBox.shrink()));
    await settleVideoPlayerTasks(tester);
    expect(port.capacitySnapshot.retiring, 1);
    expect(port.capacitySnapshot.isQuiescent, isFalse);

    platform.releaseDisposal();
    await settleVideoPlayerTasks(tester);
    expect(port.capacitySnapshot, emptyVideoPlaybackCapacitySnapshot);
    expect(port.capacitySnapshot.isQuiescent, isTrue);
  });
}
