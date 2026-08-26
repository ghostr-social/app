part of 'video_player_playback_port.dart';

extension _VideoPlayerSurfaceCapabilityRecovery on _VideoPlayerSurfaceState {
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
      widget.recoveryPolicy.initializationTimeout,
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
