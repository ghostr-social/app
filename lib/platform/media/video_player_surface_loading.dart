part of 'video_player_playback_port.dart';

extension _VideoPlayerSurfaceLoading on _VideoPlayerSurfaceState {
  Future<void> _loadController() async {
    final controller = _videoPlayerController(widget.media);
    _controller = controller;
    try {
      await controller.setLooping(true);
      if (!await _initializeUntilClosing(controller)) return;
      _requireVisibleVideo(controller);
      await _acceptController(controller);
    } on Object catch (error, stackTrace) {
      _logInitializationFailure(error, stackTrace);
      await _rejectController(controller);
    }
  }

  Future<bool> _initializeUntilClosing(VideoPlayerController controller) async {
    if (_isClosing) return false;
    final initialization = controller.initialize();
    await Future.any<void>([initialization, _closing.future]);
    if (_isClosing) return false;
    await initialization;
    return true;
  }

  Future<void> _acceptController(VideoPlayerController controller) async {
    if (!_ownsController(controller)) {
      await _disposeSafely(controller);
      return;
    }
    await _schedulePlayback(controller);
    if (!_ownsController(controller)) return;
    _valueWatch.attach(controller);
    _refresh(() {});
  }

  bool _ownsController(VideoPlayerController controller) {
    return !_isClosing && mounted && _controller == controller;
  }

  Future<void> _rejectController(VideoPlayerController controller) async {
    _valueWatch.detach();
    _endObservation(controller.value);
    _markControllerFailed(controller);
    await _disposeSafely(controller);
  }

  void _markControllerFailed(VideoPlayerController controller) {
    if (_isClosing || !mounted || _controller != controller) return;
    _refresh(() {
      _controller = null;
      _hasError = true;
    });
  }

  void _retry() {
    if (!_hasError || !_isPlayableMedia(widget.media)) return;
    _refresh(() => _hasError = false);
    _startLoad();
  }

  void _startLoad() => _lifecycle.track(_loadController());

  void _logInitializationFailure(Object error, StackTrace stackTrace) {
    log(
      'Video player initialization failed.',
      name: 'ghostr.video.player',
      error: error,
      stackTrace: stackTrace,
    );
  }
}
