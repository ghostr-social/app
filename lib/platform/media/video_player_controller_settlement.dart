part of 'video_player_playback_port.dart';

typedef _ControllerSettlementInterest = bool Function();
typedef _ControllerSettlementCompletion =
    void Function(_ControllerTeardownOutcome outcome);

final class _ControllerSettlement {
  _ControllerSettlement(this._timeout, this._dispose, this._onSettled);

  final Duration _timeout;
  final Future<void> Function() _dispose;
  final _ControllerSettlementCompletion _onSettled;
  final Completer<_ControllerTeardownOutcome> _completion = Completer();
  final Set<_ControllerSettlementDemand> _demands = {};
  Timer? _deadline;
  var _started = false;
  var _settled = false;

  bool get isSettled => _settled;
  Future<_ControllerTeardownOutcome> get outcome => _completion.future;
  Future<void> get done => outcome.then<void>((_) {});

  void start() {
    if (_started) return;
    _started = true;
    unawaited(_runDisposal());
  }

  Future<_ControllerTeardownOutcome?> require({
    required Future<void> cancelled,
    required _ControllerSettlementInterest isLive,
  }) {
    _pruneDemands();
    if (!isLive()) return Future.value();
    if (_settled) return outcome.then((value) => value);
    final demand = _ControllerSettlementDemand(isLive);
    _demands.add(demand);
    _armDeadline();
    unawaited(cancelled.then((_) => demand.abandon()));
    return _waitForDemand(demand);
  }

  Future<_ControllerTeardownOutcome?> _waitForDemand(
    _ControllerSettlementDemand demand,
  ) {
    final settled = outcome.then<_ControllerTeardownOutcome?>((value) => value);
    final abandoned = demand.abandoned.then<_ControllerTeardownOutcome?>(
      (_) => null,
    );
    return Future.any([
      settled,
      abandoned,
    ]).whenComplete(() => _cancelDemand(demand));
  }

  Future<void> _runDisposal() async {
    final exit = await _attemptDisposal();
    if (_settled) {
      _logLateDisposal(exit);
      if (exit == _ControllerTeardownExit.proven) {
        _notify(_ControllerTeardownOutcome.proven);
      }
      return;
    }
    final outcome = exit == _ControllerTeardownExit.proven
        ? _ControllerTeardownOutcome.proven
        : _ControllerTeardownOutcome.unproven;
    _settle(outcome);
  }

  Future<_ControllerTeardownExit> _attemptDisposal() async {
    try {
      await _dispose();
      return _ControllerTeardownExit.proven;
    } on Object catch (error, stackTrace) {
      log(
        'Video player teardown failed.',
        name: 'ghostr.video.player',
        error: error,
        stackTrace: stackTrace,
      );
      return _ControllerTeardownExit.failed;
    }
  }

  void _armDeadline() {
    if (_deadline != null || _settled) return;
    _deadline = Timer(_timeout, _expire);
  }

  void _expire() {
    _deadline = null;
    _pruneDemands();
    if (_demands.isEmpty || _settled) return;
    log('Video player teardown timed out.', name: 'ghostr.video.player');
    _settle(_ControllerTeardownOutcome.unproven);
  }

  void _pruneDemands() {
    final stale = _demands.where((demand) => !demand.isLive()).toList();
    for (final demand in stale) {
      _cancelDemand(demand);
    }
  }

  void _cancelDemand(_ControllerSettlementDemand demand) {
    if (!_demands.remove(demand)) return;
    demand.abandon();
    if (_demands.isNotEmpty) return;
    _deadline?.cancel();
    _deadline = null;
  }

  void _settle(_ControllerTeardownOutcome outcome) {
    if (_settled) return;
    _settled = true;
    _deadline?.cancel();
    _deadline = null;
    _demands.clear();
    _notify(outcome);
    _completion.complete(outcome);
  }

  void _notify(_ControllerTeardownOutcome outcome) {
    try {
      _onSettled(outcome);
    } on Object catch (error, stackTrace) {
      log(
        'Video player teardown settlement failed.',
        name: 'ghostr.video.player',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }

  void _logLateDisposal(_ControllerTeardownExit exit) {
    log(
      'Video player teardown settled after its deadline: ${exit.name}.',
      name: 'ghostr.video.player',
    );
  }
}

final class _ControllerSettlementDemand {
  _ControllerSettlementDemand(this.isLive);

  final _ControllerSettlementInterest isLive;
  final Completer<void> _abandoned = Completer();

  Future<void> get abandoned => _abandoned.future;

  void abandon() {
    if (!_abandoned.isCompleted) _abandoned.complete();
  }
}

enum _ControllerTeardownExit { proven, failed }
