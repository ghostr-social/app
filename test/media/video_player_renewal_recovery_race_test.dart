import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/playback_authority_fixture.dart';
import '../support/recovering_video_player_platform.dart';
import '../support/scripted_progressive_playback_refresh.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('prepared renewal cancels an older scheduled recovery', (
    tester,
  ) async {
    final platform = RecoveringVideoPlayerPlatform();
    final refresh = ScriptedProgressivePlaybackRefresh();
    final scope = VideoPlaybackSurfaceScope();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort(
      recoveryPolicy: PlaybackRecoveryPolicy([
        const Duration(milliseconds: 50),
      ]),
    );

    await pumpVideoPlayerSurface(
      tester,
      port,
      _request(scope, _firstUrl, _authority(_firstCapability), refresh),
    );
    platform.failLatest('old capability superseded');
    await tester.pump();
    await pumpVideoPlayerSurface(
      tester,
      port,
      _request(scope, _renewedUrl, _authority(_renewedCapability), refresh),
    );
    await tester.pump(const Duration(milliseconds: 100));
    await settleVideoPlayerTasks(tester);

    expect(refresh.requestCount, 0);
    expect(platform.dataSources, hasLength(2));
    expect(platform.dataSources.last.uri, _renewedUrl);
  });
}

VideoPlaybackSurfaceRequest _request(
  VideoPlaybackSurfaceScope scope,
  String url,
  PlaybackAssetAuthority authority,
  ScriptedProgressivePlaybackRefresh refresh,
) {
  return VideoPlaybackSurfaceRequest(
    media: ProxiedProgressiveVideoMediaSource(url),
    isActive: true,
    surfaceScope: scope,
    authority: authority,
    progressiveRefresh: refresh,
  );
}

PlaybackAssetAuthority _authority(String capability) {
  final original = testPlaybackAuthority();
  return PlaybackAssetAuthority(
    deliveryId: original.deliveryId,
    representationId: original.representationId,
    assetId: PlaybackAssetId.parse(capability),
  );
}

const _firstCapability = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const _renewedCapability = 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
const _firstUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap=$_firstCapability';
const _renewedUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap=$_renewedCapability';
