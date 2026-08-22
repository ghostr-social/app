part of 'video_player_playback_port.dart';

extension _VideoPlayerControllerBudgetPressure on _VideoPlayerControllerBudget {
  void _syncPressure() {
    if (!_hasWantedWaiters) {
      _cancelPressure();
      return;
    }
    _pressureCancellation ??= Completer<void>();
    for (final permit in _outstanding) {
      final retirement = permit._retirement;
      if (retirement != null) _armPressure(retirement);
    }
  }

  void _armPressure(_ControllerSettlement settlement) {
    final cancellation = _pressureCancellation;
    if (cancellation == null || _pressured.containsKey(settlement)) return;
    _pressured[settlement] = cancellation;
    unawaited(_pressureSettlement(settlement, cancellation));
  }

  Future<void> _pressureSettlement(
    _ControllerSettlement settlement,
    Completer<void> cancellation,
  ) async {
    await settlement.require(
      cancelled: cancellation.future,
      isLive: () =>
          identical(_pressureCancellation, cancellation) && _hasWantedWaiters,
    );
    if (identical(_pressured[settlement], cancellation)) {
      _pressured.remove(settlement);
    }
  }

  void _cancelPressure() {
    final cancellation = _pressureCancellation;
    _pressureCancellation = null;
    _pressured.clear();
    if (cancellation != null && !cancellation.isCompleted) {
      cancellation.complete();
    }
  }

  bool get _hasWantedWaiters {
    return _waiters.any((waiter) => waiter.wanted());
  }
}
