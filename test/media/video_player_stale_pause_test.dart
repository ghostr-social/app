import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/audited_video_player_platform.dart';

void main() {
  testWidgets('a stale pause cannot stop reactivated playback', (tester) async {
    final platform = AuditedVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort();
    final media = VideoMediaSource.local('/cache/video.mp4');
    final pause = platform.pauseGate = Completer<void>();
    addTearDown(() {
      if (!pause.isCompleted) pause.complete();
    });

    await _show(tester, port, media: media);
    await _show(tester, port, media: media, mode: VideoPlaybackMode.paused);
    await _show(tester, port, media: media);
    pause.complete();
    await tester.pump();

    expect(platform.isPlaying(0), isTrue, reason: '${platform.commands}');
  });
}

Future<void> _show(
  WidgetTester tester,
  VideoPlayerPlaybackPort port, {
  VideoMediaSource? media,
  VideoPlaybackMode mode = VideoPlaybackMode.normal,
}) async {
  final request = VideoPlaybackSurfaceRequest(
    media: media ?? VideoMediaSource.local('/cache/video.mp4'),
    isActive: true,
    mode: mode,
  );
  await tester.pumpWidget(MaterialApp(home: port.buildSurface(request)));
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}
