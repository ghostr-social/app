part of 'warp_malformed_range_scenario.dart';

extension _MalformedRangeWait on _MalformedRangeScenario {
  Future<void> waitForRejectedRange(WidgetTester tester) async {
    final watch = Stopwatch()..start();
    while (watch.elapsed < const Duration(seconds: 15)) {
      if (_hasRejectedRange()) return;
      await journey.pumpFor(tester, const Duration(milliseconds: 70));
    }
    await journey.reportSchedulingEvidence();
    fail('Malformed range was not rejected without exposing bytes.');
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

  Future<PlaybackFocus> navigatePastRejectedVideo(WidgetTester tester) async {
    final cursor = journey.focusCursor;
    await journey.swipeUp(tester);
    await journey.waitForCaption(tester, 1);
    await journey.waitForPublishedFocus(tester, 1, afterSequence: cursor);
    expect(journey.focus.hadTransportRescue, isFalse);
    expectRejectedRangeIsNotReady();
    await journey.swipeUp(tester);
    final focus = await journey.waitForPublishedFocus(
      tester,
      2,
      afterSequence: cursor,
      cause: FeedFocusCause.userNavigation,
    );
    await journey.waitForCaption(tester, 2);
    await journey.waitForFirstFrame(tester, focus);
    await journey.waitForPlaying(tester, focus);
    return focus;
  }
}
