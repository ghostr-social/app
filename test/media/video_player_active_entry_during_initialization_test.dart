import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/audited_video_player_platform.dart';
import '../support/playback_authority_fixture.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('active entry replaces a pending speculative controller', (
    tester,
  ) async {
    final platform = AuditedVideoPlayerPlatform(autoInitialize: false);
    final scope = VideoPlaybackSurfaceScope();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort();

    await pumpVideoPlayerSurface(tester, port, _request(scope, false));
    await tester.pump(const Duration(seconds: 3));
    expect(platform.createdCount, 1);

    await pumpVideoPlayerSurface(tester, port, _request(scope, true));
    await settleVideoPlayerTasks(tester);
    expect(platform.createdCount, 2);
    expect(platform.playerCount, 1);

    platform.initialize(1);
    await settleVideoPlayerTasks(tester);
    expect(find.byType(Texture), findsOneWidget);
    expect(platform.commands, contains('play:1'));
    expect(platform.commands, isNot(contains('play:0')));
    expect(find.text('Video unavailable'), findsNothing);
  });
}

VideoPlaybackSurfaceRequest _request(
  VideoPlaybackSurfaceScope scope,
  bool active,
) => VideoPlaybackSurfaceRequest(
  media: ProxiedProgressiveVideoMediaSource(
    'http://127.0.0.1:3210/video.mp4?id=warm&cap=$testPlaybackCapability',
  ),
  videoId: PlaybackVideoId.parse('warm'),
  authority: testPlaybackAuthority(postId: 'warm'),
  isActive: active,
  surfaceScope: scope,
  keepWarmWhenInactive: !active,
  reservesPreparedDecoder: true,
);
