part of 'video_player_playback_port.dart';

extension _VideoPlayerPlaybackHandoffCommand on _VideoPlayerPlaybackHandoff {
  Future<_HandoffCommandOutcome> _command(
    VideoPlayerController controller,
    Future<void> Function() operation, {
    required _HandoffCommandEffect effect,
    _HandoffActivation? activation,
  }) async {
    final state = _state(controller);
    if (state.retiring) return _HandoffCommandOutcome.disposed;
    final native = Future<void>.sync(operation);
    if (_increasesAudibility(effect)) state.holdUnsafeCommand(native);
    final races = _commandRaces(native, state, activation);
    try {
      final outcome = await Future.any(races);
      if (outcome == _HandoffCommandOutcome.cancelled) {
        _abandonCommand(controller, native, effect, activation!);
      }
      return outcome;
    } on Object {
      if (_increasesAudibility(effect)) activation?.abandonUnsafe();
      rethrow;
    }
  }

  List<Future<_HandoffCommandOutcome>> _commandRaces(
    Future<void> native,
    _HandoffControllerState state,
    _HandoffActivation? activation,
  ) {
    final races = <Future<_HandoffCommandOutcome>>[
      native.then((_) => _HandoffCommandOutcome.completed),
      state.retirement.future.then((_) => _HandoffCommandOutcome.disposed),
    ];
    if (activation != null) {
      races.add(
        activation.cancelled.then((_) => _HandoffCommandOutcome.cancelled),
      );
    }
    return races;
  }

  void _abandonCommand(
    VideoPlayerController controller,
    Future<void> native,
    _HandoffCommandEffect effect,
    _HandoffActivation activation,
  ) {
    if (_increasesAudibility(effect)) activation.abandonUnsafe();
    _observeAbandonedCommand(controller, native, effect);
  }

  bool _increasesAudibility(_HandoffCommandEffect effect) {
    return effect == _HandoffCommandEffect.play ||
        effect == _HandoffCommandEffect.unmute;
  }

  Future<void> _schedule(Future<void> Function() operation) {
    final scheduled = _tail.then((_) => operation());
    _tail = scheduled.then<void>((_) {}, onError: (_, __) {});
    return scheduled;
  }
}
