part of 'video_player_playback_port.dart';

extension _VideoPlayerSurfaceCommands on _VideoPlayerSurfaceState {
  void _syncPlayback() {
    final controller = _controller;
    if (controller == null || !controller.value.isInitialized) return;
    _lifecycle.track(_schedulePlayback(controller));
  }

  Future<void> _schedulePlayback(VideoPlayerController controller) {
    final intent = ++_playbackIntent;
    final active = widget.isActive;
    final operation = _playbackTail.then(
      (_) => _guardPlayback(controller, active, intent),
    );
    _playbackTail = operation;
    return operation;
  }

  Future<void> _guardPlayback(
    VideoPlayerController controller,
    bool active,
    int intent,
  ) async {
    try {
      await _applyPlayback(controller, active, intent);
    } on Object catch (error, stackTrace) {
      log(
        'Video playback command failed.',
        name: 'ghostr.video.player',
        error: error,
        stackTrace: stackTrace,
      );
      await _rejectController(controller);
    }
  }

  Future<void> _applyPlayback(
    VideoPlayerController controller,
    bool active,
    int intent,
  ) async {
    if (!_ownsPlaybackIntent(controller, active, intent)) return;
    if (active) {
      _beginObservation(controller.value);
      await controller.play();
      return;
    }
    _endObservation(controller.value);
    await controller.pause();
    if (_ownsPlaybackIntent(controller, active, intent)) {
      await controller.seekTo(Duration.zero);
    }
  }

  bool _ownsPlaybackIntent(
    VideoPlayerController controller,
    bool active,
    int intent,
  ) {
    return _ownsController(controller) &&
        _playbackIntent == intent &&
        widget.isActive == active;
  }
}
