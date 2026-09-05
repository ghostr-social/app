part of 'warp_cache_pressure_scenario.dart';

extension _WarpCachePressureStorage on _WarpCachePressureDriver {
  Directory get _supportDirectory => Directory(session.resources.cachePath);

  Future<WarpCacheCoverage> _cacheCoverage() {
    return readWarpCacheCoverage(_supportDirectory);
  }

  Future<void> _waitForColdCoverage() {
    return _waitAsync(() async {
      final coverage = await _cacheCoverage();
      _expectWithinBudget(coverage);
      return coverage.bytesFor(coldDelivery.value) == origin.objectLength;
    });
  }

  Future<void> _driveUntilColdEvicted() async {
    while (forwardHandoffs < _maximumPressureHandoffs) {
      final focus = await _swipeForward();
      await _waitForCompleteCoverage(focus);
      forwardHandoffs += 1;
      final coverage = await _cacheCoverage();
      _expectWithinBudget(coverage);
      if (coverage.bytesFor(coldDelivery.value) == 0) return;
    }
    final coverage = await _cacheCoverage();
    fail('Cold cache entry survived $forwardHandoffs handoffs: $coverage');
  }

  Future<void> _waitForCompleteCoverage(PlaybackFocus focus) {
    final delivery = graph.focus.deliveryForEvent(focus.videoId.value)!;
    return _waitAsync(() async {
      final coverage = await _cacheCoverage();
      _expectWithinBudget(coverage);
      return coverage.bytesFor(delivery.value) == origin.objectLength;
    });
  }

  void _expectWithinBudget(WarpCacheCoverage coverage) {
    expect(
      coverage.totalBytes,
      lessThanOrEqualTo(_cachePressureBudgetBytes),
      reason: '${coverage.byDelivery}',
    );
  }
}
