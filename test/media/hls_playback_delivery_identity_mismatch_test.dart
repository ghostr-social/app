import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/hls_video_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import '../support/fake_hls_playback_gateway.dart';
import '../support/fake_media_ports.dart';

void main() {
  testWidgets('rejects a lease for a different delivery identity', (
    tester,
  ) async {
    final gateway = FakeHlsPlaybackGateway();
    final delegate = FakeVideoPlaybackPort();
    final playback = HlsVideoPlaybackPort(delegate: delegate, gateway: gateway);
    final media = VideoMediaSource.remote(
      'https://media.test/root.m3u8',
      delivery: VideoMediaDelivery.hls,
    );

    await tester.pumpWidget(
      MaterialApp(
        home: playback.buildSurface(
          VideoPlaybackSurfaceRequest(media: media, isActive: true),
        ),
      ),
    );
    gateway.completeNext(
      deliveryId: PlaybackDeliveryId.parse('different-delivery'),
    );
    await tester.pump();

    expect(find.text('Video unavailable'), findsOneWidget);
    expect(delegate.requests, isEmpty);
    expect(gateway.activeLeaseCount, 0);
  });
}
