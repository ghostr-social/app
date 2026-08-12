import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/recovering_video_player_platform.dart';
import '../support/scripted_progressive_playback_refresh.dart';

void main() {
  testWidgets('a stale capability cannot resurrect superseded playback', (
    tester,
  ) async {
    final platform = RecoveringVideoPlayerPlatform();
    final refresh = ScriptedProgressivePlaybackRefresh();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort(
      recoveryPolicy: PlaybackRecoveryPolicy([Duration.zero]),
    );
    await _pump(
      tester,
      port,
      ProxiedProgressiveVideoMediaSource(_firstUrl),
      refresh,
    );

    platform.failLatest('source interrupted');
    await tester.pump();
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    expect(refresh.requestCount, 1);

    await _pump(tester, port, VideoMediaSource.local('/cache/next.mp4'), null);
    refresh.completeNext(_renewedUrl);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(platform.dataSources, hasLength(2));
    expect(platform.dataSources.last.uri, contains('/cache/next.mp4'));
  });
}

Future<void> _pump(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  VideoMediaSource media,
  ScriptedProgressivePlaybackRefresh? refresh,
) async {
  await tester.pumpWidget(
    MaterialApp(
      home: port.buildSurface(
        VideoPlaybackSurfaceRequest(
          media: media,
          isActive: true,
          progressiveRefresh: refresh,
        ),
      ),
    ),
  );
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}

const _firstUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap='
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const _renewedUrl =
    'http://127.0.0.1:3210/video.mp4?id=post-1&cap='
    'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
