part of 'video_player_playback_port.dart';

extension _VideoPlayerSurfaceLoading on _VideoPlayerSurfaceState {
  Future<void> _loadController() async {
    final controller = _videoPlayerController(_playbackMedia);
    _controller = controller;
    try {
      await controller.setLooping(true);
      if (!await _initializeUntilClosing(controller)) return;
      _requireVisibleVideo(controller);
      await _restorePlayhead(controller);
      await _acceptController(controller);
    } on _InvisibleVideoTrack catch (error, stackTrace) {
      _logInitializationFailure(error, stackTrace);
      await _rejectControllerPermanently(controller);
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
    await _recoverController(controller);
  }

  void _retry() {
    if (!_hasError || !_isPlayableMedia(widget.media)) return;
    _retryPlayback();
  }

  void _startLoad() => _lifecycle.track(_loadController());

  PlaybackSession? _openPlaybackSession() {
    final videoId = widget.videoId;
    final deliveryId = _playbackMedia.playbackDeliveryId;
    if (videoId == null || deliveryId == null) return null;
    return widget.telemetry.openSession(videoId, deliveryId);
  }

  void _logInitializationFailure(Object error, StackTrace stackTrace) {
    log(
      'Video player initialization failed.',
      name: 'ghostr.video.player',
      error: error,
      stackTrace: stackTrace,
    );
  }
}
