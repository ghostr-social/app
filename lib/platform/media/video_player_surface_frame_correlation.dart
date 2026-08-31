part of 'video_player_playback_port.dart';

extension _VideoPlayerSurfaceFrameCorrelation on _VideoPlayerSurfaceState {
  RenderedFirstFrameAttempt? _beginFrameAttempt() {
    final media = _playbackMedia;
    if (media is! ProxiedHlsVideoMediaSource &&
        media is! ProxiedProgressiveVideoMediaSource) {
      return null;
    }
    try {
      return widget.renderedFirstFrames.beginAttempt();
    } on Object catch (error, stackTrace) {
      _logFrameCorrelationFailure('allocation', error, stackTrace);
      return null;
    }
  }

  void _claimPreparation(
    VideoPlayerController controller,
    PlayerPreparationAttempt? attempt,
    RenderedFirstFrameAttempt? frameAttempt,
  ) {
    _resetPresentationEvidence();
    _preparationAttempt = attempt;
    _firstFrameAttempt = frameAttempt;
    _correlatedHlsAuthority = frameAttempt == null
        ? null
        : widget.request.hlsAuthority;
    if (frameAttempt == null) return;
    try {
      frameAttempt.listen(
        () => _handleNativeFrame(controller, attempt, frameAttempt),
      );
    } on Object catch (error, stackTrace) {
      _discardFrameAttempt(frameAttempt, error, stackTrace);
    }
  }

  void _discardFrameAttempt(
    RenderedFirstFrameAttempt frameAttempt,
    Object error,
    StackTrace stackTrace,
  ) {
    if (identical(_firstFrameAttempt, frameAttempt)) {
      _firstFrameAttempt = null;
      _correlatedHlsAuthority = null;
    }
    _resetPresentationEvidence();
    _releaseFrameAttemptSafely(frameAttempt);
    _logFrameCorrelationFailure('listener', error, stackTrace);
  }

  void _releaseFrameAttemptSafely(RenderedFirstFrameAttempt frameAttempt) {
    try {
      frameAttempt.release();
    } on Object catch (error, stackTrace) {
      _logFrameCorrelationFailure('release', error, stackTrace);
    }
  }

  void _handleNativeFrame(
    VideoPlayerController controller,
    PlayerPreparationAttempt? attempt,
    RenderedFirstFrameAttempt frameAttempt,
  ) {
    if (!_ownsController(controller) ||
        !identical(_preparationAttempt, attempt) ||
        !identical(_firstFrameAttempt, frameAttempt)) {
      return;
    }
    attempt?.firstFrameRendered();
    _nativeFrameObserved = true;
    _reportHlsFirstFrame();
    _schedulePresentedFrame();
  }

  void _reportHlsFirstFrame() {
    final authority = _correlatedHlsAuthority;
    final callback = widget.request.onHlsFirstFrameRendered;
    if (_playbackMedia is! ProxiedHlsVideoMediaSource ||
        !_nativeFrameObserved ||
        !_controllerPresented ||
        _reportedHlsAuthority != null ||
        authority == null ||
        widget.request.hlsAuthority != authority ||
        callback == null ||
        widget.request.playbackDeliveryId != authority.deliveryId) {
      return;
    }
    try {
      callback(authority);
      _reportedHlsAuthority = authority;
    } on Object catch (error, stackTrace) {
      _logFrameCorrelationFailure('HLS feedback', error, stackTrace);
    }
  }

  void _revokeHlsDecodedReadiness(HlsPlaybackAuthority authority) {
    try {
      widget.request.onHlsDecodedReadinessRevoked?.call(authority);
    } on Object catch (error, stackTrace) {
      _logFrameCorrelationFailure('HLS revocation', error, stackTrace);
    }
  }
}

void _logFrameCorrelationFailure(
  String operation,
  Object error,
  StackTrace stackTrace,
) {
  log(
    'Native first-frame correlation $operation failed.',
    name: 'ghostr.video.first_frame',
    error: error,
    stackTrace: stackTrace,
  );
}
