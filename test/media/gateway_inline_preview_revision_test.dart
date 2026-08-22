import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/inline_blurhash.dart';
import 'package:ghostr/core/media/video_media_metadata.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/gateway_video_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/shared/widgets/inline_blurhash_preview.dart';

import '../support/fake_media_ports.dart';
import '../support/fake_progressive_playback_gateway.dart';

void main() {
  testWidgets('gateway replaces same-identity preview and forwards it', (
    tester,
  ) async {
    final gateway = FakeProgressivePlaybackGateway();
    final delegate = FakeVideoPlaybackPort();
    final playback = GatewayVideoPlaybackPort(
      delegate: delegate,
      gateway: gateway,
    );

    await tester.pumpWidget(_app(playback, _media('000000')));
    expect(_preview(tester), '000000');

    await tester.pumpWidget(_app(playback, _media('00TI:j')));
    expect(_preview(tester), '00TI:j');
    gateway.completeNext();
    await tester.pump();
    expect(delegate.requests.single.preview?.encoded, '00TI:j');
  });
}

Widget _app(VideoPlaybackPort playback, VideoMediaSource media) {
  return MaterialApp(
    home: playback.buildSurface(
      VideoPlaybackSurfaceRequest(media: media, isActive: true),
    ),
  );
}

VideoMediaSource _media(String hash) => VideoMediaSource.remote(
  'https://media.test/clip.mp4',
  metadata: VideoMediaMetadata(blurhash: InlineBlurHash.parse(hash)),
);

String _preview(WidgetTester tester) => tester
    .widget<InlineBlurHashPreview>(find.byType(InlineBlurHashPreview))
    .descriptor
    .encoded;
