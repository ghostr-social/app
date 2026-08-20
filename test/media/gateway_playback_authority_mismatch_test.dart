import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/gateway_video_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import '../support/fake_progressive_playback_gateway.dart';
import '../support/recording_video_playback_port.dart';

void main() {
  testWidgets('gateway rejects authority for a different delivery', (
    tester,
  ) async {
    final delegate = RecordingVideoPlaybackPort();
    final port = GatewayVideoPlaybackPort(
      delegate: delegate,
      gateway: FakeProgressivePlaybackGateway(
        immediatePlaybackUrl:
            'http://127.0.0.1:4040/video.mp4?id=post-2&cap='
            'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
      ),
    );
    final origin = VideoMediaSource.withCacheScope(
      VideoMediaSource.remote('https://media.test/clip.mp4'),
      'post-1',
    );

    await tester.pumpWidget(
      MaterialApp(
        home: port.buildSurface(
          VideoPlaybackSurfaceRequest(media: origin, isActive: true),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(delegate.requests.single.authority, isNull);
  });
}
