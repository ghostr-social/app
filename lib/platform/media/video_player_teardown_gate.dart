part of 'video_player_playback_port.dart';

extension _VideoPlayerSurfaceTeardownGate on _VideoPlayerSurfaceState {
  Future<bool> _canAcquireAfterTeardown(Completer<void> cancellation) async {
    final cancelled = Future.any([_closing.future, cancellation.future]);
    final requirement = await _lifecycle.requireControllers(
      cancelled: cancelled,
      isLive: () => !_isClosing && !cancellation.isCompleted,
    );
    if (requirement == _ControllerTeardownRequirement.unproven) {
      _exhaustAfterUnprovenTeardown();
      await Future.any([_lifecycle.proofRecovered, cancelled]);
      if (_isClosing || cancellation.isCompleted) return false;
      _refresh(() => _recoveryState = _VideoPlayerRecoveryState.ready);
      return _canAcquireAfterTeardown(cancellation);
    }
    return requirement == _ControllerTeardownRequirement.proven &&
        !_isClosing &&
        !cancellation.isCompleted;
  }

  void _exhaustAfterUnprovenTeardown() {
    if (!_isClosing) {
      _refresh(() => _recoveryState = _VideoPlayerRecoveryState.exhausted);
    }
  }

  void _clearPendingCancellation(Completer<void> cancellation) {
    if (_pendingLoadCancellation == cancellation) {
      _pendingLoadCancellation = null;
    }
  }
}
