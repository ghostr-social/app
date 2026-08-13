import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';

void main() {
  testWidgets('covering active feed playback releases its platform player', (
    tester,
  ) async {
    final platform = FakeVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final port = VideoPlayerPlaybackPort(
      controllerDisposer: (controller) async {
        await controller.pause();
        await platform.dispose(controller.playerId);
      },
    );
    final media = VideoMediaSource.local('/cache/current.mp4');

    await _show(tester, port, media, isActive: true);
    expect(platform.calls, contains('play'));

    await _show(tester, port, media, isActive: false);
    await tester.pump();

    expect(
      platform.calls.where((call) => call == 'dispose'),
      hasLength(1),
      reason: '${platform.calls}',
    );
    expect(platform.calls, isNot(contains('seekTo')));
  });
}

Future<void> _show(
  WidgetTester tester,
  VideoPlayerPlaybackPort port,
  VideoMediaSource media, {
  required bool isActive,
}) async {
  await tester.pumpWidget(
    MaterialApp(
      home: port.buildSurface(
        VideoPlaybackSurfaceRequest(media: media, isActive: isActive),
      ),
    ),
  );
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}
