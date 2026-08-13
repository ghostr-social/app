import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/audited_video_player_platform.dart';

void main() {
  testWidgets('prepared next player never overlaps audible current playback', (
    tester,
  ) async {
    final platform = AuditedVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort();

    await _showPair(tester, port, activeIndex: 1);
    expect(platform.playerCount, 2);

    platform.pauseGate = Completer<void>();
    await _showPair(tester, port, activeIndex: 0, settle: false);
    await tester.pump();

    expect(platform.audibleOverlap, isFalse, reason: '${platform.commands}');
    platform.pauseGate!.complete();
    await tester.pump();
  });

  testWidgets('competing active surfaces remain single-audible', (
    tester,
  ) async {
    final platform = AuditedVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort();

    await tester.pumpWidget(
      MaterialApp(
        home: Stack(
          children: List.generate(2, (index) {
            return port.buildSurface(
              VideoPlaybackSurfaceRequest(
                media: VideoMediaSource.local('/cache/$index.mp4'),
                isActive: true,
              ),
            );
          }),
        ),
      ),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(platform.audibleOverlap, isFalse, reason: '${platform.commands}');
    expect(
      platform.commands.where((command) => command == 'volume:0:0.0'),
      isNotEmpty,
    );
  });
}

Future<void> _showPair(
  WidgetTester tester,
  VideoPlayerPlaybackPort port, {
  required int activeIndex,
  bool settle = true,
}) async {
  await tester.pumpWidget(
    MaterialApp(
      home: Stack(
        children: List.generate(2, (index) {
          return port.buildSurface(
            VideoPlaybackSurfaceRequest(
              media: VideoMediaSource.local('/cache/$index.mp4'),
              isActive: index == activeIndex,
            ),
          );
        }),
      ),
    ),
  );
  await tester.pump();
  if (settle) await tester.pump(const Duration(milliseconds: 100));
}
