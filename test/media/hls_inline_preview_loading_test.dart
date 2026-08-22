import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/inline_blurhash.dart';
import 'package:ghostr/core/media/video_media_metadata.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/hls_video_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/widgets/inline_blurhash_preview.dart';

import '../support/fake_hls_playback_gateway.dart';
import '../support/fake_media_ports.dart';

void main() {
  testWidgets('HLS acquisition renders and forwards the inline preview', (
    tester,
  ) async {
    final gateway = FakeHlsPlaybackGateway();
    final delegate = FakeVideoPlaybackPort();
    final playback = HlsVideoPlaybackPort(delegate: delegate, gateway: gateway);
    final media = VideoMediaSource.remote(
      'https://media.test/root.m3u8',
      delivery: VideoMediaDelivery.hls,
      metadata: VideoMediaMetadata(blurhash: InlineBlurHash.parse('000000')),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: playback.buildSurface(
          VideoPlaybackSurfaceRequest(media: media, isActive: true),
        ),
      ),
    );
    expect(find.byType(InlineBlurHashPreview), findsOneWidget);
    expect(find.bySemanticsLabel('Loading video'), findsOneWidget);

    gateway.completeNext();
    await tester.pump();
    expect(delegate.requests.single.preview?.encoded, '000000');
  });
}
