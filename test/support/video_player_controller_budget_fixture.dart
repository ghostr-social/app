import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import 'feed_preparation_video_player_platform.dart';

final class VideoPlayerControllerBudgetFixture {
  VideoPlayerControllerBudgetFixture({
    VideoPlayerControllerDisposer? disposer,
    PlaybackRecoveryPolicy recoveryPolicy =
        const PlaybackRecoveryPolicy.standard(),
    bool autoInitialize = true,
  }) : platform = FeedPreparationVideoPlayerPlatform(
         autoInitialize: autoInitialize,
       ),
       port = VideoPlayerPlaybackPort(
         controllerDisposer: disposer ?? disposeVideoPlayerController,
         recoveryPolicy: recoveryPolicy,
       ) {
    VideoPlayerPlatform.instance = platform;
  }

  final FeedPreparationVideoPlayerPlatform platform;
  final VideoPlayerPlaybackPort port;
  var releaseCount = 0;

  Future<void> show(
    WidgetTester tester,
    List<String> ids, {
    String active = 'b',
  }) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Stack(children: ids.map((id) => _surface(id, active)).toList()),
      ),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
  }

  Future<void> turn(WidgetTester tester) async {
    await tester.runAsync(() => Future<void>.delayed(Duration.zero));
    await tester.pump();
  }

  int creations(String id) {
    return platform.sources.values
        .where((source) => source.uri?.endsWith('/$id.mp4') == true)
        .length;
  }

  Widget _surface(String id, String active) {
    return port.buildSurface(
      VideoPlaybackSurfaceRequest(
        media: VideoMediaSource.local('/cache/$id.mp4'),
        isActive: id == active,
        onPlaybackMediaReleased: () => releaseCount += 1,
      ),
    );
  }
}
