part of 'video_player_playback_port.dart';

typedef VideoPlayerControllerDisposer = Future<void> Function(
  VideoPlayerController controller,
);

Future<void> disposeVideoPlayerController(VideoPlayerController controller) =>
    controller.dispose();

final class _VideoPlayerControllerLifecycle {
  _VideoPlayerControllerLifecycle(this._disposeController);

  final VideoPlayerControllerDisposer _disposeController;
  final Set<Future<void>> _pending = {};
  final Map<VideoPlayerController, Future<void>> _disposals = {};

  void track(Future<void> operation) {
    _pending.add(operation);
    unawaited(operation.whenComplete(() => _pending.remove(operation)));
  }

  Future<void> close() async {
    while (_pending.isNotEmpty) {
      await Future.wait(_pending.toList());
    }
  }

  Future<void> dispose(VideoPlayerController controller) {
    final pending = _disposals[controller];
    if (pending != null) return pending;
    final completion = Completer<void>();
    _disposals[controller] = completion.future;
    unawaited(_completeDisposal(controller, completion));
    return completion.future;
  }

  Future<void> _completeDisposal(
    VideoPlayerController controller,
    Completer<void> completion,
  ) async {
    try {
      await _disposeController(controller);
    } on Object catch (error, stackTrace) {
      log('Video player teardown failed.',
          name: 'ghostr.video.player', error: error, stackTrace: stackTrace);
    } finally {
      _disposals.remove(controller);
      completion.complete();
    }
  }
}
