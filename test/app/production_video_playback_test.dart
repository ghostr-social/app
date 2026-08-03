import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/app/production_video_playback.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/platform/media/hls_video_playback_port.dart';
import 'package:ghostr/platform/media/inventory_video_playback_port.dart';

import '../support/fake_hls_playback_gateway.dart';
import '../support/fake_remote_video_source.dart';
import '../support/fake_video_inventory.dart';

void main() {
  test('wraps inventory playback with HLS acquisition when available', () {
    final delivery = ProductionVideoDelivery(
      FakeVideoInventory(),
      FakeRemoteVideoSource([]),
      hlsPlaybackGateway: FakeHlsPlaybackGateway(),
      playbackCapabilities: VideoPlaybackCapabilities.progressiveAndHls,
    );

    final playback = buildProductionVideoPlayback(delivery);

    expect(playback, isA<HlsVideoPlaybackPort>());
  });

  test('uses inventory playback directly without a secure HLS gateway', () {
    final delivery = ProductionVideoDelivery(
      FakeVideoInventory(),
      FakeRemoteVideoSource([]),
    );

    final playback = buildProductionVideoPlayback(delivery);

    expect(playback, isA<InventoryVideoPlaybackPort>());
  });

  test('does not trust an HLS gateway without player capability', () {
    final delivery = ProductionVideoDelivery(
      FakeVideoInventory(),
      FakeRemoteVideoSource([]),
      hlsPlaybackGateway: FakeHlsPlaybackGateway(),
    );

    final playback = buildProductionVideoPlayback(delivery);

    expect(playback, isA<InventoryVideoPlaybackPort>());
  });

  testWidgets('renders a stable surface when the platform has no player',
      (tester) async {
    final delivery = ProductionVideoDelivery(
      FakeVideoInventory(),
      FakeRemoteVideoSource([]),
      playbackCapabilities: VideoPlaybackCapabilities.none,
    );
    final playback = buildProductionVideoPlayback(delivery);
    var releases = 0;

    await tester.pumpWidget(
      MaterialApp(
        home: playback.buildSurface(
          media: VideoMediaSource.local('/videos/draft.mp4'),
          isActive: true,
          onPlaybackMediaReleased: () => releases += 1,
        ),
      ),
    );

    expect(find.text('Video playback unavailable'), findsOneWidget);
    expect(find.text('This platform has no compatible video player.'),
        findsOneWidget);
    expect(releases, 0);

    await tester.pumpWidget(const MaterialApp(home: SizedBox.shrink()));

    expect(releases, 1);
  });
}
