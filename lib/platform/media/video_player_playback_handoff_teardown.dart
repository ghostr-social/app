part of 'video_player_playback_port.dart';

extension _VideoPlayerPlaybackHandoffTeardown on _VideoPlayerPlaybackHandoff {
  void _demandAudibleTeardown(_HandoffActivation activation) {
    final previous = _audible;
    if (previous == null) return;
    final state = _state(previous);
    final settlement = state.teardown;
    if (!state.retiring || settlement == null) return;
    unawaited(
      settlement.require(
        cancelled: activation.cancelled,
        isLive: () => activation.isLive,
      ),
    );
  }

  Future<bool> _mutePrevious(
    VideoPlayerController target,
    _HandoffActivation activation,
  ) async {
    while (activation.isLive) {
      if (await _muteOnePrevious(target, activation)) return true;
    }
    return false;
  }

  Future<bool> _muteOnePrevious(
    VideoPlayerController target,
    _HandoffActivation activation,
  ) async {
    final previous = _audible;
    if (previous == null || previous == target) return true;
    final state = _state(previous);
    if (state.retiring) {
      final outcome = await state.teardown!.require(
        cancelled: activation.cancelled,
        isLive: () => activation.isLive,
      );
      if (outcome == null) return false;
      _requireProven(outcome);
      return false;
    }
    final outcome = await _command(
      previous,
      () => previous.setVolume(0),
      effect: _HandoffCommandEffect.mute,
      activation: activation,
    );
    if (outcome != _HandoffCommandOutcome.completed) return false;
    if (_audible == previous) _audibleIsReady = false;
    return true;
  }

  void _requireProven(_ControllerTeardownOutcome outcome) {
    if (outcome == _ControllerTeardownOutcome.unproven) {
      throw StateError('Prior video player teardown was not proven.');
    }
  }
}
