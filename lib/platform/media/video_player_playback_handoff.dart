part of 'video_player_playback_port.dart';

typedef _PlaybackOwnership = bool Function();

final class _VideoPlayerPlaybackHandoff {
  Future<void> _tail = Future<void>.value();
  final Expando<_HandoffControllerState> _controllers = Expando();
  VideoPlayerController? _audible;
  bool _audibleIsReady = false;

  Future<void> activate(
    VideoPlayerController controller,
    _PlaybackOwnership ownsPlayback,
    _UnsafePlaybackAbandon onUnsafeAbandon,
  ) {
    final activation = _state(
      controller,
    ).beginActivation(ownsPlayback, onUnsafeAbandon);
    _demandAudibleTeardown(activation);
    return _schedule(() => _activate(controller, activation));
  }

  Future<void> deactivate(VideoPlayerController controller) {
    _state(controller).cancelActivation();
    return _schedule(() => _deactivate(controller));
  }

  void supersede(VideoPlayerController controller) {
    _state(controller).cancelActivation();
  }

  void retire(
    VideoPlayerController controller,
    _ControllerSettlement settlement,
  ) {
    _state(controller).retire(settlement);
  }

  void markTeardown(
    VideoPlayerController controller,
    _ControllerTeardownOutcome outcome,
  ) {
    final state = _state(controller)..retiring = true;
    if (outcome == _ControllerTeardownOutcome.proven &&
        _audible == controller) {
      _clearAudible();
    }
    state.cancelActivation();
  }

  Future<void> waitUnsafeCommands(VideoPlayerController controller) {
    return _state(controller).waitUnsafeCommands();
  }

  Future<void> _activate(
    VideoPlayerController controller,
    _HandoffActivation activation,
  ) async {
    if (!activation.isLive) return;
    if (_canKeepPlaying(controller)) return;
    if (_needsVolumeRestore(controller)) {
      await _restoreVolume(controller, activation);
      return;
    }
    if (!await _prepareMutedTarget(controller, activation)) return;
    await _playMutedTarget(controller, activation);
  }

  Future<bool> _prepareMutedTarget(
    VideoPlayerController controller,
    _HandoffActivation activation,
  ) async {
    if (!await _mutePrevious(controller, activation)) return false;
    if (!_canActivate(controller, activation)) return false;
    final outcome = await _command(
      controller,
      () => controller.setVolume(0),
      effect: _HandoffCommandEffect.mute,
      activation: activation,
    );
    if (outcome != _HandoffCommandOutcome.completed) return false;
    if (!_canActivate(controller, activation)) return false;
    _audible = controller;
    _audibleIsReady = false;
    return true;
  }

  Future<void> _playMutedTarget(
    VideoPlayerController controller,
    _HandoffActivation activation,
  ) async {
    final outcome = await _command(
      controller,
      controller.play,
      effect: _HandoffCommandEffect.play,
      activation: activation,
    );
    if (outcome != _HandoffCommandOutcome.completed) return;
    await _restoreVolume(controller, activation);
  }

  Future<void> _deactivate(VideoPlayerController controller) async {
    final mute = await _command(
      controller,
      () => controller.setVolume(0),
      effect: _HandoffCommandEffect.mute,
    );
    if (mute != _HandoffCommandOutcome.completed) return;
    if (_audible == controller) _clearAudible();
    await _command(
      controller,
      controller.pause,
      effect: _HandoffCommandEffect.pause,
    );
  }

  bool _canActivate(
    VideoPlayerController controller,
    _HandoffActivation activation,
  ) {
    return activation.isLive && !_state(controller).retiring;
  }

  bool _canKeepPlaying(VideoPlayerController controller) {
    return _audible == controller &&
        _audibleIsReady &&
        controller.value.isPlaying;
  }

  bool _needsVolumeRestore(VideoPlayerController controller) {
    return _audible == controller && controller.value.isPlaying;
  }

  Future<void> _restoreVolume(
    VideoPlayerController controller,
    _HandoffActivation activation,
  ) async {
    if (!activation.isLive) return;
    final outcome = await _command(
      controller,
      () => controller.setVolume(1),
      effect: _HandoffCommandEffect.unmute,
      activation: activation,
    );
    if (outcome != _HandoffCommandOutcome.completed) return;
    if (_canRestore(controller, activation)) _audibleIsReady = true;
  }

  bool _canRestore(
    VideoPlayerController controller,
    _HandoffActivation activation,
  ) {
    return _audible == controller && _canActivate(controller, activation);
  }

  void _clearAudible() {
    _audible = null;
    _audibleIsReady = false;
  }

  _HandoffControllerState _state(VideoPlayerController controller) {
    return _controllers[controller] ??= _HandoffControllerState();
  }
}
