import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:video_player/video_player.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import '../support/fake_video_player_platform.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('default controller disposal awaits platform teardown', () async {
    final platform = FakeVideoPlayerPlatform();
    VideoPlayerPlatform.instance = platform;
    final controller = VideoPlayerController.file(File('/cache/video.mp4'));
    await controller.initialize();
    final barrier = Completer<void>();
    platform.disposeBarrier = barrier;

    var completed = false;
    final disposal = disposeVideoPlayerController(controller).then((_) {
      completed = true;
    });
    await platform.disposeStarted.future;
    expect(completed, isFalse);

    barrier.complete();
    await disposal;
    expect(completed, isTrue);
  });
}
