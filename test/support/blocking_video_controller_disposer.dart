import 'dart:async';

import 'package:video_player/video_player.dart';

class BlockingVideoControllerDisposer {
  final Completer<void> started = Completer<void>();
  final Completer<void> release = Completer<void>();

  Future<void> call(VideoPlayerController controller) {
    unawaited(controller.pause());
    if (!started.isCompleted) started.complete();
    return release.future;
  }
}
