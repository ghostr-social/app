part of 'video_player_playback_port.dart';

typedef VideoPlayerControllerDisposer =
    Future<void> Function(VideoPlayerController controller);
typedef _ControllerTeardownStarted =
    void Function(
      VideoPlayerController controller,
      _ControllerSettlement settlement,
    );
typedef _ControllerTeardownCompletion =
    void Function(
      VideoPlayerController controller,
      _ControllerTeardownOutcome outcome,
    );

enum _ControllerTeardownRequirement { proven, unproven, cancelled }

Future<void> disposeVideoPlayerController(VideoPlayerController controller) =>
    controller.dispose();

final class _VideoPlayerControllerLifecycle {
  _VideoPlayerControllerLifecycle(
    this._disposeController,
    this._onTeardownStarted,
    this._onTeardown,
    this._teardownTimeout,
  );

  final VideoPlayerControllerDisposer _disposeController;
  final _ControllerTeardownStarted _onTeardownStarted;
  final _ControllerTeardownCompletion _onTeardown;
  final Duration _teardownTimeout;
  final Expando<_ControllerSettlement> _disposals = Expando();
  final Set<_ControllerSettlement> _pendingDisposals = {};
  final Map<VideoPlayerController, _ControllerPermit> _permits = {};
  final Set<VideoPlayerController> _unproven = {};
  Completer<void>? _proofRecovery;

  Future<void> get proofRecovered => _unproven.isEmpty
      ? Future<void>.value()
      : (_proofRecovery ??= Completer<void>()).future;

  void attach(VideoPlayerController controller, _ControllerPermit permit) {
    assert(!_permits.containsKey(controller));
    _permits[controller] = permit;
  }

  void track(Future<void> operation) => unawaited(_observe(operation));

  Future<bool> waitControllers() async {
    while (true) {
      final pending = _pendingSettlements();
      if (pending.isEmpty) return _unproven.isEmpty;
      await Future.wait(pending.map((settlement) => settlement.outcome));
    }
  }

  Future<_ControllerTeardownRequirement> requireControllers({
    required Future<void> cancelled,
    required _ControllerSettlementInterest isLive,
  }) async {
    while (isLive()) {
      final pending = _pendingSettlements();
      if (pending.isEmpty) return _settledRequirement;
      final outcomes = await Future.wait(
        pending.map(
          (settlement) =>
              settlement.require(cancelled: cancelled, isLive: isLive),
        ),
      );
      if (outcomes.any((outcome) => outcome == null)) {
        return _ControllerTeardownRequirement.cancelled;
      }
    }
    return _ControllerTeardownRequirement.cancelled;
  }

  _ControllerTeardownRequirement get _settledRequirement {
    return _unproven.isNotEmpty
        ? _ControllerTeardownRequirement.unproven
        : _ControllerTeardownRequirement.proven;
  }

  Future<void> dispose(VideoPlayerController controller) {
    final existing = _disposals[controller];
    if (existing != null) return existing.done;
    final permit = _permits.remove(controller);
    late final _ControllerSettlement settlement;
    settlement = _ControllerSettlement(
      _teardownTimeout,
      () => _disposeController(controller),
      (outcome) => _finishDisposal(controller, settlement, permit, outcome),
    );
    _disposals[controller] = settlement;
    _pendingDisposals.add(settlement);
    permit?.retire(settlement);
    try {
      _onTeardownStarted(controller, settlement);
    } finally {
      settlement.start();
    }
    return settlement.done;
  }

  List<_ControllerSettlement> _pendingSettlements() {
    return _pendingDisposals
        .where((settlement) => !settlement.isSettled)
        .toList();
  }

  void _finishDisposal(
    VideoPlayerController controller,
    _ControllerSettlement settlement,
    _ControllerPermit? permit,
    _ControllerTeardownOutcome outcome,
  ) {
    if (outcome == _ControllerTeardownOutcome.unproven) {
      _unproven.add(controller);
    } else {
      _unproven.remove(controller);
      if (_unproven.isEmpty) {
        _proofRecovery?.complete();
        _proofRecovery = null;
      }
    }
    try {
      _onTeardown(controller, outcome);
    } finally {
      try {
        if (outcome == _ControllerTeardownOutcome.proven) {
          permit?.release();
        } else {
          permit?.quarantine();
        }
      } finally {
        _pendingDisposals.remove(settlement);
      }
    }
  }

  Future<void> _observe(Future<void> operation) async {
    try {
      await operation;
    } on Object catch (error, stackTrace) {
      log(
        'Video player lifecycle operation failed.',
        name: 'ghostr.video.player',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}
