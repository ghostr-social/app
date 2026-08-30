import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
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

  testWidgets('rejects every mismatched prepared HLS authority field', (
    tester,
  ) async {
    final media = VideoMediaSource.withCacheScope(
      VideoMediaSource.remote(
        'https://media.test/root.m3u8',
        delivery: VideoMediaDelivery.hls,
      ),
      'post-A',
    );
    final expected = _authority(media);
    final mismatches = [
      _authority(media, delivery: 'post-B'),
      _authority(media, representation: 'b' * 64),
      _authority(media, revision: 2),
    ];
    for (final mismatch in mismatches) {
      final gateway = FakeHlsPlaybackGateway();
      final delegate = FakeVideoPlaybackPort();
      final playback = HlsVideoPlaybackPort(
        delegate: delegate,
        gateway: gateway,
      );
      await tester.pumpWidget(
        MaterialApp(
          key: UniqueKey(),
          home: playback.buildSurface(
            VideoPlaybackSurfaceRequest(
              media: media,
              isActive: true,
              hlsAuthority: expected,
            ),
          ),
        ),
      );
      gateway.completeNext(authority: mismatch);
      await tester.pump();

      expect(find.text('Video unavailable'), findsOneWidget);
      expect(delegate.requests, isEmpty);
      expect(gateway.activeLeaseCount, 0);
    }
  });
}

HlsPlaybackAuthority _authority(
  VideoMediaSource media, {
  String delivery = 'post-A',
  String? representation,
  int revision = 1,
}) {
  return HlsPlaybackAuthority(
    deliveryId: PlaybackDeliveryId.parse(delivery),
    representationId: representation == null
        ? VideoRepresentationId.forMedia(media)
        : VideoRepresentationId.parse(representation),
    assetRevision: HlsPlaybackAssetRevision.parse(BigInt.from(revision)),
  );
}
