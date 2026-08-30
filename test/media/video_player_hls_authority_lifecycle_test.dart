import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/rendered_first_frame_protocol.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';
import '../support/recording_player_preparation_feedback.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('HLS frame verification is exact and revoked with its player', (
    tester,
  ) async {
    final platform = FakeVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final events = StreamController<Object?>();
    final frames = NativeRenderedFirstFramePort(events: events.stream);
    final feedback = RecordingPlayerPreparationFeedback();
    final verified = <HlsPlaybackAuthority>[];
    final released = <HlsPlaybackAuthority>[];
    final port = VideoPlayerPlaybackPort(
      preparationFeedback: feedback,
      renderedFirstFrames: frames,
    );
    addTearDown(() async {
      await frames.dispose();
      await events.close();
    });
    final stale = _authority(1);
    final current = _authority(2);

    await pumpVideoPlayerSurface(
      tester,
      port,
      _request('a', stale, verified, released),
    );
    await settleVideoPlayerTasks(tester);
    final staleToken = _token(platform, 0);
    await pumpVideoPlayerSurface(
      tester,
      port,
      _request('b', current, verified, released),
    );
    await settleVideoPlayerTasks(tester);
    final currentToken = _token(platform, 1);

    events.add({'version': 1, 'attemptToken': staleToken});
    await settleVideoPlayerTasks(tester);
    expect(verified, isEmpty);
    events.add({'version': 1, 'attemptToken': currentToken});
    await settleVideoPlayerTasks(tester);
    expect(verified, [current]);
    expect(feedback.events, isEmpty);

    await tester.pumpWidget(const SizedBox());
    await settleVideoPlayerTasks(tester);
    expect(released, [current]);
  });
}

VideoPlaybackSurfaceRequest _request(
  String session,
  HlsPlaybackAuthority authority,
  List<HlsPlaybackAuthority> verified,
  List<HlsPlaybackAuthority> released,
) => VideoPlaybackSurfaceRequest(
  media: ProxiedHlsVideoMediaSource(
    'http://127.0.0.1:3210/hls/${session * 64}/index.m3u8',
  ),
  isActive: true,
  playbackDeliveryId: authority.deliveryId,
  hlsAuthority: authority,
  onHlsFirstFrameRendered: verified.add,
  onHlsDecodedReadinessRevoked: released.add,
);

HlsPlaybackAuthority _authority(int revision) => HlsPlaybackAuthority(
  deliveryId: PlaybackDeliveryId.parse('post-A'),
  representationId: VideoRepresentationId.parse('a' * 64),
  assetRevision: HlsPlaybackAssetRevision.parse(BigInt.from(revision)),
);

String _token(FakeVideoPlayerPlatform platform, int index) {
  return platform.dataSources[index].httpHeaders[warpPlaybackAttemptHeader]!;
}
