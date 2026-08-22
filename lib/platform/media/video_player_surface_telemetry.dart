part of 'video_player_playback_port.dart';

extension _VideoPlayerSurfaceTelemetry on _VideoPlayerSurfaceState {
  void _beginObservation(VideoPlayerValue value) {
    if (_isObserving) return;
    _isObserving = true;
    _playbackObserver.reset();
    final session = _playbackSession ??= _openPlaybackSession();
    if (session != null) widget.telemetry.activate(session);
    _captureObservation(value);
    _schedulePresentedFrame();
  }

  bool _captureObservation(VideoPlayerValue value) {
    if (!_isObserving || !widget.isActive) return false;
    if (!value.isInitialized && !value.hasError) return false;
    final previous = _playbackPhase;
    final phase = _playbackObserver.classify(
      value,
      isActive: true,
      isUserPaused: widget.mode == VideoPlaybackMode.paused,
    );
    _playbackPhase = phase;
    widget.screenAwake.observePhase(this, phase);
    _reportObservation(value, phase);
    return phase != previous;
  }

  void _endObservation(VideoPlayerValue? value) {
    if (!_isObserving) return;
    if (value != null && value.isInitialized) {
      final phase = _playbackObserver.classify(value, isActive: false);
      _playbackPhase = phase;
      _reportObservation(value, phase);
    }
    final session = _playbackSession;
    _isObserving = false;
    widget.screenAwake.release(this);
    if (session != null) widget.telemetry.deactivate(session);
  }

  void _reportObservation(VideoPlayerValue value, PlaybackPhase phase) {
    final session = _playbackSession;
    if (session == null) return;
    widget.telemetry.report(_playbackObserver.observe(session, value, phase));
  }
}
