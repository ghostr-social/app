part of 'video_player_playback_port.dart';

extension _VideoPlayerSurfaceCommands on _VideoPlayerSurfaceState {
  void _leaveActivePlayback() {
    if (widget.request.keepWarmWhenInactive) {
      _retainedWarmController = _controller != null;
      _endObservation(_controller?.value);
      _playbackSession = null;
      _playbackPhase = null;
      _resetActivationPresentation();
      _syncPlayback();
      return;
    }
    _coverPlayback();
  }

  void _coverPlayback() {
    _retainedWarmController = false;
    _loadRequested = false;
    _cancelPendingLoad();
    final controller = _controller;
    if (controller == null) return;
    _playbackIntent += 1;
    _rememberPlaybackValue(controller.value);
    _endObservation(controller.value);
    _valueWatch.detach();
    _relinquishController(controller);
    _playbackSession = null;
    _playbackPhase = null;
    _refresh(() {});
    _lifecycle.track(_disposeSafely(controller));
  }

  void _syncPlayback() {
    final controller = _controller;
    if (controller == null || !controller.value.isInitialized) return;
    _lifecycle.track(_schedulePlayback(controller));
  }

  Future<void> _schedulePlayback(VideoPlayerController controller) {
    final active = widget.isActive;
    final mode = widget.mode;
    if (!active || !mode.shouldPlay) widget.handoff.supersede(controller);
    final intent = ++_playbackIntent;
    final operation = _playbackTail.then(
      (_) => _guardPlayback(controller, active, mode, intent),
    );
    _playbackTail = operation;
    return operation;
  }

  Future<void> _guardPlayback(
    VideoPlayerController controller,
    bool active,
    VideoPlaybackMode mode,
    int intent,
  ) async {
    try {
      await _applyPlayback(controller, active, mode, intent);
    } on Object catch (error, stackTrace) {
      log(
        'Video playback command failed.',
        name: 'ghostr.video.player',
        error: error,
        stackTrace: stackTrace,
      );
      _failPreparation(PlayerPreparationFailureKind.playbackCommand);
      await _rejectController(controller);
    }
  }

  Future<void> _applyPlayback(
    VideoPlayerController controller,
    bool active,
    VideoPlaybackMode mode,
    int intent,
  ) async {
    if (!_ownsPlaybackIntent(controller, active, mode, intent)) return;
    if (!active) {
      await _deactivatePlayback(controller, active, mode, intent);
      return;
    }
    await _applyActiveMode(controller, active, mode, intent);
  }

  Future<void> _applyActiveMode(
    VideoPlayerController controller,
    bool active,
    VideoPlaybackMode mode,
    int intent,
  ) async {
    _beginObservation(controller.value);
    if (!mode.shouldPlay) {
      await _pausePlayback(controller, active, mode, intent);
      return;
    }
    if (mode == VideoPlaybackMode.normal) {
      await _setPlaybackSpeedSafely(controller, mode.speed);
    }
    if (!_ownsPlaybackIntent(controller, active, mode, intent)) return;
    await widget.handoff.activate(
      controller,
      () => _ownsPlaybackIntent(controller, active, mode, intent),
      () => _abandonUnsafeController(controller),
    );
    if (mode == VideoPlaybackMode.accelerated &&
        _ownsPlaybackIntent(controller, active, mode, intent)) {
      await _setPlaybackSpeedSafely(controller, mode.speed);
    }
  }

  void _abandonUnsafeController(VideoPlayerController controller) {
    _failPreparation(PlayerPreparationFailureKind.playbackCommand);
    _lifecycle.track(_rejectController(controller));
  }

  Future<void> _pausePlayback(
    VideoPlayerController controller,
    bool active,
    VideoPlaybackMode mode,
    int intent,
  ) async {
    await _setPlaybackSpeedSafely(controller, VideoPlaybackMode.normal.speed);
    if (_ownsPlaybackIntent(controller, active, mode, intent)) {
      await widget.handoff.deactivate(controller);
    }
  }

  Future<void> _deactivatePlayback(
    VideoPlayerController controller,
    bool active,
    VideoPlaybackMode mode,
    int intent,
  ) async {
    final preservesPreparedPosition =
        _retainedWarmController || widget.request.reservesPreparedDecoder;
    _endObservation(controller.value);
    await widget.handoff.deactivate(controller);
    if (_ownsPlaybackIntent(controller, active, mode, intent) &&
        !preservesPreparedPosition) {
      await controller.seekTo(Duration.zero);
    }
  }

  Future<void> _setPlaybackSpeedSafely(
    VideoPlayerController controller,
    double speed,
  ) async {
    try {
      await controller.setPlaybackSpeed(speed);
    } on Object catch (error, stackTrace) {
      _logSpeedFailure(speed, error, stackTrace);
      if (speed != VideoPlaybackMode.normal.speed) {
        await _restoreNormalSpeed(controller);
      }
    }
  }

  Future<void> _restoreNormalSpeed(VideoPlayerController controller) async {
    try {
      await controller.setPlaybackSpeed(VideoPlaybackMode.normal.speed);
    } on Object catch (error, stackTrace) {
      _logSpeedFailure(VideoPlaybackMode.normal.speed, error, stackTrace);
    }
  }

  void _logSpeedFailure(double speed, Object error, StackTrace stackTrace) {
    log(
      'Video playback speed $speed was unavailable.',
      name: 'ghostr.video.player',
      error: error,
      stackTrace: stackTrace,
    );
  }

  bool _ownsPlaybackIntent(
    VideoPlayerController controller,
    bool active,
    VideoPlaybackMode mode,
    int intent,
  ) {
    return _ownsController(controller) &&
        _playbackIntent == intent &&
        widget.isActive == active &&
        widget.mode == mode;
  }
}
