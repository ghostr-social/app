import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

Future<void> pumpVideoPlayerSurface(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  VideoPlaybackSurfaceRequest request,
) async {
  await tester.pumpWidget(MaterialApp(home: port.buildSurface(request)));
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}
