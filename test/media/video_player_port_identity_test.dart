import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/audited_video_player_platform.dart';
import '../support/video_player_surface_pump.dart';

void main() {
  testWidgets('replacing the playback port replaces its native controller', (
    tester,
  ) async {
    final platform = AuditedVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final request = VideoPlaybackSurfaceRequest(
      media: VideoMediaSource.local('/cache/a.mp4'),
      isActive: true,
    );
    await _show(tester, VideoPlayerPlaybackPort(), request);
    await _show(tester, VideoPlayerPlaybackPort(), request);

    final created = platform.createdCount;
    final live = platform.playerCount;
    final replacementPlays = platform.isPlaying(1);
    await tester.pumpWidget(const SizedBox());
    await settleVideoPlayerTasks(tester);
    expect(created, 2);
    expect(live, 1);
    expect(replacementPlays, isTrue);
  });
}

Future<void> _show(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  VideoPlaybackSurfaceRequest request,
) async {
  await tester.pumpWidget(MaterialApp(home: port.buildSurface(request)));
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
  await settleVideoPlayerTasks(tester);
}
