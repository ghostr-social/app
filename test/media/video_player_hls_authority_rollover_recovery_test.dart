import 'dart:async';

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
  testWidgets('HLS authority rollover replaces and verifies the decoder', (
    tester,
  ) async {
    final platform = FakeVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final events = StreamController<Object?>();
    final frames = NativeRenderedFirstFramePort(events: events.stream);
    final feedback = RecordingPlayerPreparationFeedback();
    final verified = <HlsPlaybackAuthority>[];
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

    await pumpVideoPlayerSurface(tester, port, _request(stale, verified));
    await settleVideoPlayerTasks(tester);
    await pumpVideoPlayerSurface(tester, port, _request(current, verified));
    await settleVideoPlayerTasks(tester);

    expect(platform.dataSources, hasLength(2));
    expect(feedback.hlsStatesFor(stale), [
      RecordedPreparationState.initializing,
      RecordedPreparationState.initialized,
      RecordedPreparationState.released,
    ]);
    events.add(_frame(platform, 0));
    await settleVideoPlayerTasks(tester);
    expect(verified, isEmpty);
    events.add(_frame(platform, 1));
    await settleVideoPlayerTasks(tester);
    expect(verified, [current]);
    expect(feedback.hlsStatesFor(current), [
      RecordedPreparationState.initializing,
      RecordedPreparationState.initialized,
      RecordedPreparationState.firstFrameRendered,
    ]);
  });
}

VideoPlaybackSurfaceRequest _request(
  HlsPlaybackAuthority authority,
  List<HlsPlaybackAuthority> verified,
) => VideoPlaybackSurfaceRequest(
  media: ProxiedHlsVideoMediaSource(_playbackUrl),
  isActive: true,
  playbackDeliveryId: authority.deliveryId,
  hlsAuthority: authority,
  onHlsFirstFrameRendered: verified.add,
);

HlsPlaybackAuthority _authority(int revision) => HlsPlaybackAuthority(
  deliveryId: PlaybackDeliveryId.parse('post-A'),
  representationId: VideoRepresentationId.parse('a' * 64),
  assetRevision: HlsPlaybackAssetRevision.parse(BigInt.from(revision)),
);

Map<String, Object> _frame(FakeVideoPlayerPlatform platform, int index) => {
  'version': 1,
  'attemptToken':
      platform.dataSources[index].httpHeaders[warpPlaybackAttemptHeader]!,
};

const _playbackUrl =
    'http://127.0.0.1:3210/hls/'
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/'
    'index.m3u8';
