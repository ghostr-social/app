import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_inventory/domain/rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/audited_video_player_platform.dart';
import '../support/drain_test_microtasks.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('early HLS frame cannot verify a newer authority revision', (
    tester,
  ) async {
    final platform = AuditedVideoPlayerPlatform(autoInitialize: false);
    VideoPlayerPlatform.instance = platform;
    final events = StreamController<Object?>();
    final token = RenderedFirstFrameAttemptToken.parse(_token);
    final frames = NativeRenderedFirstFramePort(
      events: events.stream,
      tokenFactory: () => token,
    );
    final verified = <HlsPlaybackAuthority>[];
    final port = VideoPlayerPlaybackPort(renderedFirstFrames: frames);
    addTearDown(() async {
      await frames.dispose();
      await events.close();
    });

    await pumpVideoPlayerSurface(
      tester,
      port,
      _request(_authority(1), verified),
    );
    events.add({'version': 1, 'attemptToken': _token});
    await tester.runAsync(drainTestMicrotasks);
    await pumpVideoPlayerSurface(
      tester,
      port,
      _request(_authority(2), verified),
    );
    platform.initialize(0);
    await settleVideoPlayerTasks(tester);

    expect(verified, isEmpty);
  });
}

VideoPlaybackSurfaceRequest _request(
  HlsPlaybackAuthority authority,
  List<HlsPlaybackAuthority> verified,
) => VideoPlaybackSurfaceRequest(
  media: ProxiedHlsVideoMediaSource(_playbackUrl),
  isActive: false,
  playbackDeliveryId: authority.deliveryId,
  reservesPreparedDecoder: true,
  hlsAuthority: authority,
  onHlsFirstFrameRendered: verified.add,
);

HlsPlaybackAuthority _authority(int revision) => HlsPlaybackAuthority(
  deliveryId: PlaybackDeliveryId.parse('post-A'),
  representationId: VideoRepresentationId.parse('a' * 64),
  assetRevision: HlsPlaybackAssetRevision.parse(BigInt.from(revision)),
);

const _playbackUrl =
    'http://127.0.0.1:3210/hls/'
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/'
    'index.m3u8';

const _token = 'abcdefghijklmnopqrstuA';
