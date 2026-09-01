part of 'video_player_playback_port.dart';

extension _VideoPlayerSurfaceActivity on _VideoPlayerSurfaceState {
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
    if (!widget.isActive) {
      _leaveActivePlayback();
      return;
    }
    if (_restartPendingProgressiveOnActiveEntry()) return;
    if (_controller == null &&
        _recoveryState == _VideoPlayerRecoveryState.ready) {
      _startLoad();
      return;
    }
    _activateReadyPlayback();
  }

  bool _restartPendingProgressiveOnActiveEntry() {
    final controller = _controller;
    final media = _playbackMedia;
    final authority = _playbackAuthority;
    if (controller == null ||
        controller.value.isInitialized ||
        media is! ProxiedProgressiveVideoMediaSource ||
        authority == null ||
        !_proxyMatches(media, authority)) {
      return false;
    }
    _restartUninitializedPlayback();
    return true;
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
}
