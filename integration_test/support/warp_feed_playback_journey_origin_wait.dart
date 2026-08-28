part of 'warp_feed_playback_journey.dart';

typedef WarpOriginUse = ({int bytes, int requests});
typedef WarpOriginSnapshot = Map<String, WarpOriginUse>;

extension WarpFeedPlaybackJourneyOriginWait on WarpFeedPlaybackJourney {
  WarpOriginSnapshot originSnapshot(Iterable<String> ids) {
    return Map.unmodifiable({
      for (final id in ids)
        id: (
          bytes: resources.origin.bytesServed(id),
          requests: resources.origin.requests
              .where((request) => request.path == '/$id.mp4')
              .length,
        ),
    });
  }

  String originRequestEvidence(Iterable<String> ids) {
    final paths = ids.map((id) => '/$id.mp4').toSet();
    return resources.origin.requests
        .where((request) => paths.contains(request.path))
        .map((request) => _originRequestLine(resources.origin, request))
        .join('|');
  }

  String originChunkEvidence(Iterable<String> ids) {
    return ids
        .expand(resources.origin.confirmedChunkEventsFor)
        .map(
          (event) =>
              'seq=${event.requestSequence}:g=${event.profileGeneration}:'
              '${event.start}-${event.end}:service_us='
              '${event.serviceStartedAt.inMicroseconds}:sent_us='
              '${event.sentAt.inMicroseconds}:confirmed_ms='
              '${event.confirmedAtEpochMs}',
        )
        .join('|');
  }

  Future<WarpOriginSnapshot> waitForOriginQuiescence(
    WidgetTester tester,
    Iterable<String> ids,
  ) async {
    final expected = ids.toSet();
    WarpOriginSnapshot? previous;
    var stableTicks = 0;
    await _wait(tester, () {
      final current = originSnapshot(expected);
      final stable =
          !_hasServingOrigin(expected) && mapEquals(previous, current);
      stableTicks = stable ? stableTicks + 1 : 0;
      previous = current;
      return stableTicks >= 8;
    });
    return previous!;
  }

  bool _hasServingOrigin(Set<String> ids) {
    final paths = ids.map((id) => '/$id.mp4').toSet();
    return resources.origin.requests.any(
      (request) =>
          paths.contains(request.path) &&
          request.outcome == ProgressiveOriginRequestOutcome.serving,
    );
  }

  Future<void> waitForParallelRangedVideos(WidgetTester tester) async {
    try {
      await _wait(tester, () => resources.origin.hadParallelRangedVideos);
    } on Object {
      await reportSchedulingEvidence();
      rethrow;
    }
  }

  Future<void> waitForFirstChunkRendezvous(
    WidgetTester tester,
    ProgressiveOriginFirstChunkRendezvous rendezvous,
  ) async {
    await _wait(tester, () => rendezvous.isSettled);
    if (!rendezvous.timedOut) return;
    await reportSchedulingEvidence();
    fail(
      'Only ${rendezvous.arrivedPaths} began before the watchdog; '
      'origin=${_originEvidence()}.',
    );
  }

  Future<ProgressiveRangedRequestPair> waitForParallelBytes(
    WidgetTester tester,
    Iterable<String> paths,
  ) async {
    try {
      await _wait(
        tester,
        () => resources.origin.rangedByteOverlap(paths) != null,
      );
    } on Object {
      await reportSchedulingEvidence();
      rethrow;
    }
    return resources.origin.rangedByteOverlap(paths)!;
  }
}

String _originRequestLine(
  ProgressiveDeviceOrigin origin,
  ProgressiveOriginRequest request,
) {
  final range = request.range;
  final span = range == null ? 'full' : '${range.start}-${range.end}';
  return 'seq=${origin.requestSequenceFor(request)}:${request.method}:'
      '${request.path}:$span:served=${request.servedBytes}:'
      '${request.outcome.name}:time_us=${request.startedAt.inMicroseconds}/'
      '${request.firstByteAt?.inMicroseconds}/'
      '${request.lastByteAt?.inMicroseconds}/'
      '${request.finishedAt?.inMicroseconds}';
}
