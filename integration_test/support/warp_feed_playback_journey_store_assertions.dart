part of 'warp_feed_playback_journey.dart';

typedef _ReplayStoreExpectation = ({
  PlaybackDeliveryId deliveryId,
  BigInt total,
});

extension WarpFeedPlaybackJourneyStoreAssertions on WarpFeedPlaybackJourney {
  Future<void> waitForNativeStoreCoverage(
    WidgetTester tester,
    Iterable<String> ids,
  ) async {
    final expected = {for (final id in ids) id: _storeExpectation(id)};
    try {
      await _wait(tester, () => expected.values.every(_hasStoreCoverage));
    } on TestFailure {
      fail('Native store coverage timed out: ${_coverageEvidence(expected)}');
    }
  }

  void expectNativeStoreCoverage(Iterable<String> ids) {
    final expected = {for (final id in ids) id: _storeExpectation(id)};
    expect(
      expected.values.every(_hasStoreCoverage),
      isTrue,
      reason: 'Invalid native store history: ${_coverageEvidence(expected)}',
    );
  }

  _ReplayStoreExpectation _storeExpectation(String id) {
    final event = _eventForOriginId(events, id);
    final deliveryId = focus.deliveryForEvent(event.id);
    if (deliveryId == null) throw StateError('No delivery identity for $id.');
    return (
      deliveryId: deliveryId,
      total: BigInt.from(resources.origin.objectLength),
    );
  }

  bool _hasStoreCoverage(_ReplayStoreExpectation expected) {
    return warpNativeStoreHistoryIsValid(
      graph.deliveryProbe.observations.map((item) => item.snapshot),
      expected.deliveryId,
      expected.total,
    );
  }

  String _coverageEvidence(Map<String, _ReplayStoreExpectation> expected) {
    return expected.entries.map(_coverageEntryEvidence).join('; ');
  }

  String _coverageEntryEvidence(
    MapEntry<String, _ReplayStoreExpectation> entry,
  ) {
    final item = entry.value;
    final history = graph.deliveryProbe.observations
        .where(
          (observation) => observation.snapshot.deliveryId == item.deliveryId,
        )
        .toList();
    return '${entry.key}=native_total:${item.total},'
        'latest:${_observationEvidence(history.lastOrNull)},'
        'peak:${_peakBytes(history)},'
        'history:${history.reversed.take(6).map(_observationEvidence).join("|")}';
  }

  BigInt _peakBytes(List<WarpFeedDeliveryObservation> history) {
    return history.fold(
      BigInt.zero,
      (value, item) => item.snapshot.bytesPresent > value
          ? item.snapshot.bytesPresent
          : value,
    );
  }

  String _observationEvidence(WarpFeedDeliveryObservation? observation) {
    if (observation == null) return 'none';
    final snapshot = observation.snapshot;
    return '${snapshot.bytesPresent}/${snapshot.totalBytes}:'
        '${snapshot.phase.name}:seq${observation.sequence}:'
        '${observation.elapsed.inMilliseconds}ms';
  }
}
