part of 'video_player_playback_port.dart';

extension _VideoPlayerSurfaceLoading on _VideoPlayerSurfaceState {
  Future<void> _loadController() async {
    final cancellation = Completer<void>();
    _pendingLoadCancellation = cancellation;
    if (!await _canAcquireAfterTeardown(cancellation)) {
      _clearPendingCancellation(cancellation);
      return;
    }
    final permit = await _acquireControllerPermit(cancellation);
    _clearPendingCancellation(cancellation);
    if (permit == null) return;
    final preparation = _preparePreparation();
    preparation?.begin();
    final controller = _claimController(permit, preparation);
    if (controller == null) return;
    final superseded = _ownController(controller);
    _claimPreparation(controller, preparation);
    await _prepareController(controller, superseded.future);
  }

  Future<_ControllerPermit?> _acquireControllerPermit(
    Completer<void> cancellation,
  ) async {
    final acquisition = await widget.controllerBudget.acquire(
      cancelled: Future.any([_closing.future, cancellation.future]),
      wanted: () => !_isClosing && !cancellation.isCompleted,
      prioritized: () =>
          !_isClosing && !cancellation.isCompleted && widget.isActive,
    );
    return _resolveControllerAcquisition(acquisition, cancellation);
  }

  _ControllerPermit? _resolveControllerAcquisition(
    _ControllerAcquisition acquisition,
    Completer<void> cancellation,
  ) {
    if (_isClosing || cancellation.isCompleted) {
      if (acquisition case _ControllerGranted(:final permit)) permit.release();
      return null;
    }
    switch (acquisition) {
      case _ControllerGranted(:final permit):
        return permit;
      case _ControllerCancelled():
        return null;
      case _ControllerExhausted():
        _refresh(() => _recoveryState = _VideoPlayerRecoveryState.exhausted);
        return null;
    }
  }

  Future<void> _prepareController(
    VideoPlayerController controller,
    Future<void> superseded,
  ) async {
    try {
      await controller.setLooping(true);
      if (!await _initializeUntilClosing(controller, superseded)) return;
      _requireVisibleVideo(controller);
      await _restorePlayhead(controller);
      if (await _acceptController(controller)) _finishPluginInitialization();
    } on _InvisibleVideoTrack catch (error, stackTrace) {
      _logInitializationFailure(error, stackTrace);
      _failPreparation(PlayerPreparationFailureKind.invalidVideoTrack);
      await _rejectControllerPermanently(controller);
    } on Object catch (error, stackTrace) {
      await _handlePreparationFailure(controller, error, stackTrace);
    }
  }

  VideoPlayerController? _claimController(
    _ControllerPermit permit,
    PlayerPreparationAttempt? preparation,
  ) {
    try {
      final controller = _videoPlayerController(
        _playbackMedia,
        preparation?.nativeToken,
      );
      _lifecycle.attach(controller, permit);
      return controller;
    } on Object catch (error, stackTrace) {
      permit.release();
      preparation?.failed(PlayerPreparationFailureKind.initialization);
      _logInitializationFailure(error, stackTrace);
      if (!_isClosing) {
        _refresh(() => _recoveryState = _VideoPlayerRecoveryState.exhausted);
      }
      return null;
    }
  }

  Future<bool> _initializeUntilClosing(
    VideoPlayerController controller,
    Future<void> superseded,
  ) async {
    if (!_ownsController(controller)) return false;
    final initialization = controller.initialize();
    final deadline = _InitializationDeadline(
      widget.recoveryPolicy.initializationTimeout,
    );
    final exit = await deadline.wait(
      initialization: initialization,
      closed: _closing.future,
      superseded: superseded,
    );
    if (exit == _InitializationExit.timedOut) {
      _failPreparation(PlayerPreparationFailureKind.initializationTimeout);
      await _rejectController(controller);
    }
    return exit == _InitializationExit.initialized &&
        _ownsController(controller);
  }

  Completer<void> _ownController(VideoPlayerController controller) {
    assert(_controllerSuperseded == null);
    final superseded = Completer<void>();
    _controller = controller;
    _controllerSuperseded = superseded;
    return superseded;
  }

  void _relinquishController(VideoPlayerController controller) {
    if (_controller != controller) return;
    _releasePreparation();
    _controller = null;
    final superseded = _controllerSuperseded;
    _controllerSuperseded = null;
    if (superseded != null && !superseded.isCompleted) superseded.complete();
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

  void _startLoad() {
    if (_isClosing) return;
    if (_isLoading) {
      if (!_hasValidPendingLoad) _loadRequested = true;
      return;
    }
    _loadRequested = true;
    _isLoading = true;
    _lifecycle.track(_drainRequestedLoads());
  }

  bool get _hasValidPendingLoad {
    final cancellation = _pendingLoadCancellation;
    return cancellation != null && !cancellation.isCompleted;
  }

  void _cancelPendingLoad() {
    final cancellation = _pendingLoadCancellation;
    if (cancellation != null && !cancellation.isCompleted) {
      cancellation.complete();
    }
  }

  Future<void> _drainRequestedLoads() async {
    try {
      while (_loadRequested && !_isClosing) {
        _loadRequested = false;
        await _loadController();
      }
    } finally {
      _isLoading = false;
    }
  }

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
