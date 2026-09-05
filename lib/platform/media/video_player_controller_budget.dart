part of 'video_player_playback_port.dart';

typedef _ControllerInterest = bool Function();

sealed class _ControllerAcquisition {
  const _ControllerAcquisition();
}

final class _ControllerGranted extends _ControllerAcquisition {
  const _ControllerGranted(this.permit);

  final _ControllerPermit permit;
}

final class _ControllerCancelled extends _ControllerAcquisition {
  const _ControllerCancelled();
}

final class _ControllerExhausted extends _ControllerAcquisition {
  const _ControllerExhausted(this.recovered);

  final Future<void> recovered;
}

final class _VideoPlayerControllerBudget {
  _VideoPlayerControllerBudget(this.maximum) : _limit = maximum;

  final int maximum;
  int _limit;
  final List<_ControllerWaiter> _waiters = [];
  final Set<_ControllerPermit> _outstanding = {};
  final Map<_ControllerSettlement, Completer<void>> _pressured = {};
  Completer<void>? _pressureCancellation;
  Completer<void>? _capacityRecovery;
  var _inUse = 0;
  var _quarantined = 0;

  Future<_ControllerAcquisition> acquire({
    required Future<void> cancelled,
    required _ControllerInterest wanted,
    required _ControllerInterest prioritized,
  }) {
    if (!wanted()) return Future.value(const _ControllerCancelled());
    if (_isExhausted) return Future.value(_exhausted);
    if (_inUse < _limit) return Future.value(_ControllerGranted(_claim()));
    final waiter = _ControllerWaiter(wanted, prioritized);
    _waiters.add(waiter);
    unawaited(cancelled.then((_) => _cancel(waiter)));
    _syncPressure();
    return waiter.future;
  }

  _ControllerPermit _claim() {
    _inUse += 1;
    assert(_inUse <= maximum);
    return _newPermit();
  }

  _ControllerPermit _newPermit() {
    final permit = _ControllerPermit(this);
    _outstanding.add(permit);
    return permit;
  }

  void _cancel(_ControllerWaiter waiter) {
    if (!_waiters.remove(waiter)) return;
    waiter.complete(const _ControllerCancelled());
    _syncPressure();
  }

  void _release(_ControllerPermit permit, {bool wasQuarantined = false}) {
    if (wasQuarantined) _quarantined -= 1;
    _outstanding.remove(permit);
    if (!_isExhausted) {
      _capacityRecovery?.complete();
      _capacityRecovery = null;
    }
    final waiter = _inUse <= _limit ? _takeNext() : null;
    if (waiter != null) {
      waiter.complete(_ControllerGranted(_newPermit()));
      _syncPressure();
      return;
    }
    _inUse -= 1;
    assert(_inUse >= 0);
    _syncPressure();
  }

  void _quarantine(_ControllerPermit permit) {
    _outstanding.remove(permit);
    _quarantined += 1;
    assert(_quarantined <= _inUse && _inUse <= maximum);
    if (!_isExhausted) {
      _syncPressure();
      return;
    }
    _dropUnwanted();
    final waiters = List<_ControllerWaiter>.of(_waiters);
    _waiters.clear();
    for (final waiter in waiters) {
      waiter.complete(_exhausted);
    }
    _syncPressure();
  }

  void _retire(_ControllerPermit permit, _ControllerSettlement settlement) {
    assert(_outstanding.contains(permit));
    _armPressure(settlement);
  }

  bool get _isExhausted => _quarantined >= _limit;

  _ControllerExhausted get _exhausted =>
      _ControllerExhausted((_capacityRecovery ??= Completer<void>()).future);

  VideoPlaybackCapacitySnapshot get snapshot => (
    inUse: _inUse,
    outstanding: _outstanding.length,
    retiring: _outstanding.where((permit) => permit._retirement != null).length,
    waiting: _waiters.length,
    quarantined: _quarantined,
  );

  void constrainTo(int limit) {
    _limit = limit.clamp(1, maximum);
    _dropUnwanted();
    _syncPressure();
  }

  _ControllerWaiter? _takeNext() {
    _dropUnwanted();
    if (_waiters.isEmpty) return null;
    final priority = _waiters.indexWhere((waiter) => waiter.prioritized());
    return _waiters.removeAt(priority < 0 ? 0 : priority);
  }

  void _dropUnwanted() {
    for (var index = _waiters.length - 1; index >= 0; index -= 1) {
      final waiter = _waiters[index];
      if (waiter.wanted()) continue;
      _waiters.removeAt(index).complete(const _ControllerCancelled());
    }
  }
}

final class _ControllerWaiter {
  _ControllerWaiter(this.wanted, this.prioritized);

  final _ControllerInterest wanted;
  final _ControllerInterest prioritized;
  final Completer<_ControllerAcquisition> _completion = Completer();

  Future<_ControllerAcquisition> get future => _completion.future;

  void complete(_ControllerAcquisition result) => _completion.complete(result);
}

final class _ControllerPermit {
  _ControllerPermit(this._budget);

  final _VideoPlayerControllerBudget _budget;
  _ControllerSettlement? _retirement;
  var _state = _ControllerPermitState.active;

  void retire(_ControllerSettlement settlement) {
    if (_state != _ControllerPermitState.active || _retirement != null) return;
    _retirement = settlement;
    _budget._retire(this, settlement);
  }

  void release() {
    if (_state == _ControllerPermitState.released) return;
    final quarantined = _state == _ControllerPermitState.quarantined;
    _state = _ControllerPermitState.released;
    _budget._release(this, wasQuarantined: quarantined);
  }

  void quarantine() {
    if (_state != _ControllerPermitState.active) return;
    _state = _ControllerPermitState.quarantined;
    _budget._quarantine(this);
  }
}

enum _ControllerPermitState { active, quarantined, released }
