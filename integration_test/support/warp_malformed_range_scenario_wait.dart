part of 'warp_malformed_range_scenario.dart';

extension _MalformedRangeWait on _MalformedRangeScenario {
  Future<void> waitForRejectedRangeAndRescue(WidgetTester tester) async {
    final watch = Stopwatch()..start();
    while (watch.elapsed < const Duration(seconds: 15)) {
      if (_hasRejectedRange() && _hasVerifiedRescue()) return;
      await journey.pumpFor(tester, const Duration(milliseconds: 70));
    }
    await journey.reportSchedulingEvidence();
    fail('Malformed range did not yield a verified rescue candidate.');
  }

  bool _hasRejectedRange() {
    final requests = journey.resources.origin.requestsFor('next');
    final terminalRange = requests.any((request) {
      return request.method == 'GET' &&
          request.range != null &&
          request.outcome != ProgressiveOriginRequestOutcome.serving;
    });
    final snapshots = _malformedSnapshots;
    return terminalRange &&
        snapshots.isNotEmpty &&
        snapshots.last.phase != VideoDeliveryPhase.startable &&
        snapshots.last.bytesPresent == BigInt.zero;
  }

  bool _hasVerifiedRescue() {
    if (journey.preparation.observations.isEmpty) return false;
    final ready = journey.preparation.latest.upcoming.any((asset) {
      return asset.authority.deliveryId == rescueId &&
          asset.readiness == PlaybackPreparationReadiness.ready;
    });
    if (!ready) return false;
    return journey.playerStages.attemptsFor(rescueId).any((attempt) {
      return attempt.firstFrameAt != null && !attempt.isTerminal;
    });
  }

  Future<PlaybackFocus> swipeToRescue(WidgetTester tester) async {
    final cursor = journey.focusCursor;
    await journey.swipeUp(tester);
    final focus = await journey.waitForPublishedFocus(
      tester,
      2,
      afterSequence: cursor,
      cause: FeedFocusCause.transportRescue,
    );
    await journey.waitForCaption(tester, 2);
    await journey.waitForFirstFrame(tester, focus);
    await journey.waitForPlaying(tester, focus);
    return focus;
  }
}
