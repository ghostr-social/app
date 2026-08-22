part of 'video_player_playback_port.dart';

extension _VideoPlayerPlaybackHandoffReconciliation
    on _VideoPlayerPlaybackHandoff {
  void _observeAbandonedCommand(
    VideoPlayerController controller,
    Future<void> native,
    _HandoffCommandEffect effect,
  ) {
    unawaited(_finishAbandonedCommand(controller, native, effect));
  }

  Future<void> _finishAbandonedCommand(
    VideoPlayerController controller,
    Future<void> native,
    _HandoffCommandEffect effect,
  ) async {
    try {
      await native;
    } on Object catch (error, stackTrace) {
      _logAbandonedCommand(error, stackTrace);
    } finally {
      if (effect == _HandoffCommandEffect.mute) {
        _queueMuteRepair(controller);
      }
    }
  }

  void _queueMuteRepair(VideoPlayerController controller) {
    final state = _state(controller);
    if (state.retiring || !state.requestMuteRepair()) return;
    unawaited(_runMuteRepair(controller, state));
  }

  Future<void> _runMuteRepair(
    VideoPlayerController controller,
    _HandoffControllerState state,
  ) async {
    try {
      await _schedule(() => _drainMuteRepair(controller, state));
    } on Object catch (error, stackTrace) {
      _logAbandonedCommand(error, stackTrace);
    } finally {
      if (state.finishMuteRepair()) _queueMuteRepair(controller);
    }
  }

  Future<void> _drainMuteRepair(
    VideoPlayerController controller,
    _HandoffControllerState state,
  ) async {
    while (state.takeMuteRepair()) {
      await _repairLateMute(controller, state);
    }
  }

  Future<void> _repairLateMute(
    VideoPlayerController controller,
    _HandoffControllerState state,
  ) async {
    final activation = state.liveActivation;
    if (activation == null || _audible != controller || !_audibleIsReady) {
      return;
    }
    await _restoreVolume(controller, activation);
  }

  void _logAbandonedCommand(Object error, StackTrace stackTrace) {
    log(
      'An abandoned native playback command failed.',
      name: 'ghostr.video.player',
      error: error,
      stackTrace: stackTrace,
    );
  }
}
