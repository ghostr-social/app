import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/playback_authority_fixture.dart';
import '../support/recovering_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('inactive prepared player adopts a renewed exact capability', (
    tester,
  ) async {
    final platform = RecoveringVideoPlayerPlatform();
    final scope = VideoPlaybackSurfaceScope();
    final port = VideoPlayerPlaybackPort();
    VideoPlayerPlatform.instance = platform;

    await pumpVideoPlayerSurface(
      tester,
      port,
      _request(scope, _firstUrl, _authority(_firstCapability)),
    );
    await pumpVideoPlayerSurface(
      tester,
      port,
      _request(scope, _renewedUrl, _authority(_renewedCapability)),
    );
    await settleVideoPlayerTasks(tester);

    expect(platform.dataSources, hasLength(2));
    expect(platform.dataSources.last.uri, _renewedUrl);
  });
}

VideoPlaybackSurfaceRequest _request(
  VideoPlaybackSurfaceScope scope,
  String url,
  PlaybackAssetAuthority authority,
) {
  return VideoPlaybackSurfaceRequest(
    media: ProxiedProgressiveVideoMediaSource(url),
    isActive: false,
    surfaceScope: scope,
    authority: authority,
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
