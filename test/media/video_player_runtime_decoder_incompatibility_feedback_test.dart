import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/playback_authority_fixture.dart';
import '../support/recording_playback_telemetry_port.dart';
import '../support/recording_player_preparation_feedback.dart';
import '../support/recovering_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('runtime decoder incompatibility waits for renewed authority', (
    tester,
  ) async {
    final platform = RecoveringVideoPlayerPlatform(retainDisposedStreams: true);
    VideoPlayerPlatform.instance = platform;
    final feedback = RecordingPlayerPreparationFeedback();
    final telemetry = RecordingPlaybackTelemetryPort();
    final port = VideoPlayerPlaybackPort(
      preparationFeedback: feedback,
      telemetry: telemetry,
    );

    final scope = VideoPlaybackSurfaceScope();
    final authority = testPlaybackAuthority();
    await pumpVideoPlayerSurface(
      tester,
      port,
      VideoPlaybackSurfaceRequest(
        media: ProxiedProgressiveVideoMediaSource(_playbackUrl),
        videoId: PlaybackVideoId.parse('post-1'),
        isActive: true,
        authority: authority,
        surfaceScope: scope,
      ),
    );
    platform.failLatest('[VideoDecoderUnsupported] selected format');
    await settleVideoPlayerTasks(tester);

    expect(platform.dataSources, hasLength(1));
    expect(find.bySemanticsLabel('Loading video'), findsOneWidget);
    expect(find.text('Video unavailable'), findsNothing);
    await pumpVideoPlayerSurface(
      tester,
      port,
      _request(authority, scope, isActive: false),
    );
    await pumpVideoPlayerSurface(tester, port, _request(authority, scope));
    expect(platform.dataSources, hasLength(1));

    final replacement = PlaybackAssetAuthority(
      deliveryId: authority.deliveryId,
      representationId: authority.representationId,
      assetId: PlaybackAssetId.parse(_replacementCapability),
    );
    await pumpVideoPlayerSurface(tester, port, _request(replacement, scope));
    expect(platform.dataSources, hasLength(2));
    platform.fail(0, '[VideoDecoderUnsupported] stale format');
    await settleVideoPlayerTasks(tester);

    expect(
      feedback.events.where(
        (event) => event.failure?.name == 'decoderUnsupported',
      ),
      hasLength(1),
    );
    expect(find.text('Video unavailable'), findsNothing);
    expect(
      telemetry.observations.map((observation) => observation.phase),
      isNot(contains(PlaybackPhase.failed)),
    );
  });
}

VideoPlaybackSurfaceRequest _request(
  PlaybackAssetAuthority authority,
  VideoPlaybackSurfaceScope scope, {
  bool isActive = true,
}) {
  return VideoPlaybackSurfaceRequest(
    media: ProxiedProgressiveVideoMediaSource(
      'http://127.0.0.1:3210/video.mp4?id=post-1&cap=${authority.assetId.value}',
    ),
    videoId: PlaybackVideoId.parse('post-1'),
    isActive: isActive,
    authority: authority,
    surfaceScope: scope,
  );
}

const _playbackUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap='
    '$testPlaybackCapability';
const _replacementCapability = 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
