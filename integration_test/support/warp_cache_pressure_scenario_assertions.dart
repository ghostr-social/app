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
    await expectWarpRequestBounds(graph.evidence);
    final coverage = await _cacheCoverage();
    _expectWithinBudget(coverage);
    expect(origin.bytesServed('long-00'), greaterThan(coldBytesBeforeReturn));
    expect(peakMountedPlayers, lessThanOrEqualTo(2));
    expect(peakControllerCapacity, lessThanOrEqualTo(2));
    expect(unavailableWasVisible, isFalse);
    expect(activePlaceholderWasVisible, isFalse);
    _expectOriginBounded();
  }

  void _expectOriginBounded() {
    expect(origin.maximumConcurrentResponses, lessThanOrEqualTo(4));
    expect(origin.requests.length, lessThanOrEqualTo(40));
    for (final id in origin.bodyRequestedIds) {
      final coverage = origin.coverageFor(id);
      expect(coverage.isWithinObject, isTrue, reason: id);
      // A cold revisit intentionally refetches evicted bytes, at most once.
      expect(
        coverage.networkBytes,
        lessThanOrEqualTo(origin.objectLength * 2 + 64 * 1024),
        reason: id,
      );
    }
  }
}
