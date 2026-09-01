part of 'warp_cache_pressure_scenario.dart';

extension _WarpCachePressureAssertions on _WarpCachePressureDriver {
  Future<void> _expectDecodedAndAdvancing(PlaybackFocus returned) async {
    final session = graph.telemetry.probe.sessionFor(returned)!;
    expect(session.generation, greaterThan(coldPlayerGeneration));
    final before = graph.telemetry.probe.latestPositionFor(returned)!;
    await _pumpFor(const Duration(seconds: 1));
    final after = graph.telemetry.probe.latestPositionFor(returned)!;
    expect(after, greaterThan(before));
  }

  Future<void> _expectPressureContract() async {
    final coverage = await _cacheCoverage();
    _expectWithinBudget(coverage);
    expect(
      origin.requestsFor('long-00').length,
      greaterThan(coldRequestsBeforeReturn),
    );
    expect(peakMountedPlayers, lessThanOrEqualTo(8));
    expect(peakControllerCapacity, lessThanOrEqualTo(8));
    expect(unavailableWasVisible, isFalse);
    expect(activePlaceholderWasVisible, isFalse);
    _expectOriginBounded();
  }

  void _expectOriginBounded() {
    expect(origin.maximumConcurrentResponses, lessThanOrEqualTo(4));
    expect(origin.requests.length, lessThanOrEqualTo(160));
    for (final id in origin.bodyRequestedIds) {
      final coverage = origin.coverageFor(id);
      expect(coverage.isWithinObject, isTrue, reason: id);
      expect(coverage.completedDuplicateBytes, 0, reason: id);
      expect(
        coverage.duplicateBytes,
        coverage.cancellationAttributedDuplicateBytes,
        reason: id,
      );
    }
  }
}
