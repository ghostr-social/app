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
      return coverage.bytesFor(coldDelivery.value) > 0;
    });
  }

  Future<void> _driveUntilColdEvicted() async {
    while (forwardHandoffs < _maximumPressureHandoffs) {
      await _swipeForward();
      forwardHandoffs += 1;
      final coverage = await _cacheCoverage();
      _expectWithinBudget(coverage);
      if (coverage.bytesFor(coldDelivery.value) == 0) return;
    }
    final coverage = await _cacheCoverage();
    fail('Cold cache entry survived $forwardHandoffs handoffs: $coverage');
  }

  void _expectWithinBudget(WarpCacheCoverage coverage) {
    expect(
      coverage.totalBytes,
      lessThanOrEqualTo(_cachePressureBudgetBytes),
      reason: '${coverage.byDelivery}',
    );
  }
}
