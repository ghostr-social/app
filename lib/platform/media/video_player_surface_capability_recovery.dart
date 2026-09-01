part of 'video_player_playback_port.dart';

extension _VideoPlayerSurfaceCapabilityRecovery on _VideoPlayerSurfaceState {
  bool get _awaitsHlsTransportRescue =>
      _playbackMedia is ProxiedHlsVideoMediaSource &&
      widget.request.playbackDeliveryId != null &&
      widget.request.hlsAuthority == null;

  Future<void> _rejectInvisibleTrack(VideoPlayerController controller) {
    final canSelectFallback = _preparationAttempt != null;
    _failPreparation(PlayerPreparationFailureKind.invalidVideoTrack);
    return canSelectFallback
        ? _rejectControllerForCapability(controller)
        : _rejectControllerPermanently(controller);
  }

  Future<void> _rejectControllerForCapability(
    VideoPlayerController controller,
  ) {
    if (!_ownsController(controller)) return Future<void>.value();
    _detachFailedController(controller);
    _cancelRecovery();
    final version = _recoveryVersion;
    _refresh(() => _recoveryState = _VideoPlayerRecoveryState.capabilityRescue);
    _lifecycle.track(_disposeSafely(controller));
    _recoveryTimer = Timer(
      widget.recoveryPolicy.capabilityRescueTimeout,
      () => _exhaustCapabilityRescue(version),
    );
    return Future<void>.value();
  }

  void _exhaustCapabilityRescue(int version) {
    if (!_acceptsRecovery(version) ||
        _recoveryState != _VideoPlayerRecoveryState.capabilityRescue) {
      return;
    }
    _recoveryTimer = null;
    _refresh(() => _recoveryState = _VideoPlayerRecoveryState.exhausted);
  }
}
