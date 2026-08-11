part of 'video_player_playback_port.dart';

enum _VideoPlayerRecoveryState { ready, scheduled, deferred, exhausted }

extension _VideoPlayerSurfaceRecovery on _VideoPlayerSurfaceState {
  Future<void> _rejectControllerPermanently(VideoPlayerController controller) {
    if (!_ownsController(controller)) return Future<void>.value();
    _detachFailedController(controller);
    _cancelRecovery();
    _refresh(() => _recoveryState = _VideoPlayerRecoveryState.exhausted);
    _lifecycle.track(_disposeSafely(controller));
    return Future<void>.value();
  }

  Future<void> _recoverController(VideoPlayerController controller) {
    if (!_ownsController(controller)) return Future<void>.value();
    _detachFailedController(controller);
    final version = _beginRecovery();
    final decision = widget.recoveryPolicy.decide(_recoveryAttempt, _activity);
    _applyRecoveryDecision(decision);
    _lifecycle.track(_disposeSafely(controller));
    if (decision case PlaybackRecoveryScheduled(:final delay)) {
      _scheduleRecovery(delay, version);
    }
    return Future<void>.value();
  }

  void _detachFailedController(VideoPlayerController controller) {
    _rememberPlaybackValue(controller.value);
    _valueWatch.detach();
    _endObservation(controller.value);
    _controller = null;
    _playbackPhase = null;
  }

  int _beginRecovery() {
    _recoveryTimer?.cancel();
    _recoveryBaseline = _resumePoint;
    _isRecoveryWindowOpen = true;
    return ++_recoveryVersion;
  }

  void _applyRecoveryDecision(PlaybackRecoveryDecision decision) {
    switch (decision) {
      case PlaybackRecoveryScheduled():
        _refresh(() => _recoveryState = _VideoPlayerRecoveryState.scheduled);
      case PlaybackRecoveryDeferred():
        _refresh(() => _recoveryState = _VideoPlayerRecoveryState.deferred);
      case PlaybackRecoveryExhausted():
        _refresh(() => _recoveryState = _VideoPlayerRecoveryState.exhausted);
    }
  }

  void _scheduleRecovery(Duration delay, int version) {
    if (!_acceptsRecovery(version)) return;
    _recoveryTimer = Timer(delay, () => _runRecovery(version));
  }

  void _runRecovery(int version) {
    _recoveryTimer = null;
    if (!_acceptsRecovery(version)) return;
    _recoveryAttempt = _recoveryAttempt.next;
    _lifecycle.track(_refreshPlaybackMedia(version));
  }

  Future<void> _refreshPlaybackMedia(int version) async {
    try {
      final refresh = widget.progressiveRefresh;
      if (refresh != null) _playbackMedia = await refresh.refresh();
      if (!_acceptsRecovery(version)) return;
      _refresh(() => _recoveryState = _VideoPlayerRecoveryState.ready);
      _startLoad();
    } on Object catch (error, stackTrace) {
      _logRefreshFailure(error, stackTrace);
      if (_acceptsRecovery(version)) _continueAfterRefreshFailure();
    }
  }

  void _continueAfterRefreshFailure() {
    final version = ++_recoveryVersion;
    final decision = widget.recoveryPolicy.decide(_recoveryAttempt, _activity);
    _applyRecoveryDecision(decision);
    if (decision case PlaybackRecoveryScheduled(:final delay)) {
      _scheduleRecovery(delay, version);
    }
  }

  void _retryPlayback() {
    _resetRecoveryBudget();
    final version = ++_recoveryVersion;
    _refresh(() => _recoveryState = _VideoPlayerRecoveryState.scheduled);
    _lifecycle.track(_refreshPlaybackMedia(version));
  }

  void _logRefreshFailure(Object error, StackTrace stackTrace) {
    log(
      'Progressive playback capability refresh failed.',
      name: 'ghostr.video.player',
      error: error,
      stackTrace: stackTrace,
    );
  }

  void _handleActivityChange() {
    if (_recoveryState == _VideoPlayerRecoveryState.scheduled &&
        !widget.isActive) {
      _deferRecovery();
      return;
    }
    if (_recoveryState == _VideoPlayerRecoveryState.deferred &&
        widget.isActive) {
      _resumeDeferredRecovery();
      return;
    }
    _syncPlayback();
  }

  void _deferRecovery() {
    _cancelRecovery();
    _refresh(() => _recoveryState = _VideoPlayerRecoveryState.deferred);
  }

  void _resumeDeferredRecovery() {
    final version = ++_recoveryVersion;
    final decision = widget.recoveryPolicy.decide(
      _recoveryAttempt,
      PlaybackSurfaceActivity.active,
    );
    _applyRecoveryDecision(decision);
    if (decision case PlaybackRecoveryScheduled(:final delay)) {
      _scheduleRecovery(delay, version);
    }
  }

  void _rememberPlaybackValue(VideoPlayerValue value) {
    if (!value.isInitialized || value.position.isNegative) return;
    if (_hasRecoveredBeyondBaseline(value.position)) _resetRecoveryBudget();
    _resumePoint = PlaybackResumePoint(value.position);
  }

  bool _hasRecoveredBeyondBaseline(Duration position) {
    return _isRecoveryWindowOpen && position > _recoveryBaseline.position;
  }

  void _resetRecoveryBudget() {
    _recoveryAttempt = PlaybackRecoveryAttempt.first;
    _isRecoveryWindowOpen = false;
  }

  Future<void> _restorePlayhead(VideoPlayerController controller) async {
    final target = _resumePoint.within(controller.value.duration);
    if (target == Duration.zero) return;
    await controller.seekTo(target);
  }

  PlaybackSurfaceActivity get _activity => widget.isActive
      ? PlaybackSurfaceActivity.active
      : PlaybackSurfaceActivity.inactive;

  bool _acceptsRecovery(int version) {
    return !_isClosing && mounted && version == _recoveryVersion;
  }

  void _cancelRecovery() {
    _recoveryVersion += 1;
    _recoveryTimer?.cancel();
    _recoveryTimer = null;
  }
}
