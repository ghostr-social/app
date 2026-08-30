import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/native_rendered_first_frame_port.dart';
import 'package:ghostr/platform/media/rendered_first_frame_protocol.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/recovering_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('keeps warm HLS readiness until its controller is relinquished', (
    tester,
  ) async {
    final platform = RecoveringVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final events = StreamController<Object?>();
    final frames = NativeRenderedFirstFramePort(events: events.stream);
    final verified = <HlsPlaybackAuthority>[];
    final revoked = <HlsPlaybackAuthority>[];
    final authority = _authority();
    final port = VideoPlayerPlaybackPort(
      recoveryPolicy: PlaybackRecoveryPolicy([const Duration(seconds: 1)]),
      renderedFirstFrames: frames,
    );
    addTearDown(() async {
      await frames.dispose();
      await events.close();
    });
    VideoPlaybackSurfaceRequest request(bool active) {
      return VideoPlaybackSurfaceRequest(
        media: ProxiedHlsVideoMediaSource(_playbackUrl),
        isActive: active,
        playbackDeliveryId: authority.deliveryId,
        keepWarmWhenInactive: true,
        hlsAuthority: authority,
        onHlsFirstFrameRendered: verified.add,
        onHlsDecodedReadinessRevoked: revoked.add,
      );
    }

    await pumpVideoPlayerSurface(tester, port, request(true));
    await settleVideoPlayerTasks(tester);
    final token =
        platform.dataSources.single.httpHeaders[warpPlaybackAttemptHeader];
    events.add({'version': 1, 'attemptToken': token});
    await settleVideoPlayerTasks(tester);
    expect(verified, [authority]);

    await pumpVideoPlayerSurface(tester, port, request(false));
    await settleVideoPlayerTasks(tester);
    expect(revoked, isEmpty);
    expect(platform.dataSources, hasLength(1));

    await pumpVideoPlayerSurface(tester, port, request(true));
    platform.failLatest('origin reset');
    await settleVideoPlayerTasks(tester);
    await settleVideoPlayerTasks(tester);

    expect(platform.commands, contains('dispose:0'));
    expect(revoked, [authority]);
  });
}

HlsPlaybackAuthority _authority() => HlsPlaybackAuthority(
  deliveryId: PlaybackDeliveryId.parse('post-A'),
  representationId: VideoRepresentationId.parse('a' * 64),
  assetRevision: HlsPlaybackAssetRevision.parse(BigInt.one),
);

const _playbackUrl =
    'http://127.0.0.1:3210/hls/'
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/'
    'index.m3u8';
