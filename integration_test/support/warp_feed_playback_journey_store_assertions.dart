part of 'warp_feed_playback_journey.dart';

typedef _ReplayStoreExpectation = ({
  PlaybackDeliveryId deliveryId,
  BigInt bytes,
  BigInt total,
});

extension WarpFeedPlaybackJourneyStoreAssertions on WarpFeedPlaybackJourney {
  Future<void> waitForReplayStoreCoverage(
    WidgetTester tester,
    Iterable<String> ids,
  ) async {
    final expected = {for (final id in ids) id: _storeExpectation(id)};
    try {
      await _wait(tester, () => expected.values.every(_hasStoreCoverage));
    } on TestFailure {
      fail('Replay store coverage timed out: ${_coverageEvidence(expected)}');
    }
  }

  _ReplayStoreExpectation _storeExpectation(String id) {
    final state = cubit.state as FeedLoaded;
    final post = state.posts.singleWhere((post) {
      final path = Uri.parse(post.media.remoteUrl!).path;
      return path.endsWith('/$id.mp4');
    });
    final coverage = resources.origin.coverageFor(id);
    return (
      deliveryId: post.media.playbackDeliveryId!,
      bytes: BigInt.from(coverage.uniqueBytes),
      total: BigInt.from(coverage.objectLength),
    );
  }

  bool _hasStoreCoverage(_ReplayStoreExpectation expected) {
    final snapshot = _latestDelivery(expected.deliveryId);
    return snapshot?.bytesPresent == expected.bytes &&
        snapshot?.totalBytes == expected.total;
  }

  VideoDeliverySnapshot? _latestDelivery(PlaybackDeliveryId deliveryId) {
    for (final observation in graph.deliveryProbe.observations.reversed) {
      if (observation.snapshot.deliveryId == deliveryId) {
        return observation.snapshot;
      }
    }
    return null;
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
    return '${entry.key}=expected:${item.bytes}/${item.total},'
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
