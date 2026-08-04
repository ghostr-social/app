import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_playback.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/platform/media/gateway_video_playback_port.dart';
import 'package:ghostr/platform/media/hls_video_playback_port.dart';

import '../support/fake_hls_playback_gateway.dart';
import '../support/fake_remote_video_source.dart';
import '../support/fake_video_inventory.dart';
import '../support/test_video_delivery.dart';

void main() {
  test('wraps gateway playback with HLS acquisition when available', () {
    final delivery = testVideoDelivery(
      inventory: FakeVideoInventory(),
      remoteSource: FakeRemoteVideoSource([]),
      hlsPlaybackGateway: FakeHlsPlaybackGateway(),
      playbackCapabilities: VideoPlaybackCapabilities.progressiveAndHls,
    );

    final playback = buildProductionVideoPlayback(delivery);

    expect(playback, isA<HlsVideoPlaybackPort>());
  });

  test('streams progressive playback from the loopback gateway', () {
    final delivery = testVideoDelivery(
      inventory: FakeVideoInventory(),
      remoteSource: FakeRemoteVideoSource([]),
    );

    final playback = buildProductionVideoPlayback(delivery);

    expect(playback, isA<GatewayVideoPlaybackPort>());
  });

  test('does not trust an HLS gateway without player capability', () {
    final delivery = testVideoDelivery(
      inventory: FakeVideoInventory(),
      remoteSource: FakeRemoteVideoSource([]),
      hlsPlaybackGateway: FakeHlsPlaybackGateway(),
    );

    final playback = buildProductionVideoPlayback(delivery);

    expect(playback, isA<GatewayVideoPlaybackPort>());
  });

  testWidgets('renders a stable surface when the platform has no player',
      (tester) async {
    final delivery = testVideoDelivery(
      inventory: FakeVideoInventory(),
      remoteSource: FakeRemoteVideoSource([]),
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
