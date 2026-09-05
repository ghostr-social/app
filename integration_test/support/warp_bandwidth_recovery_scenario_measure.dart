part of 'warp_bandwidth_recovery_scenario.dart';

typedef _LinkChange = ({
  String path,
  int bandwidth,
  WarpDecisionRecord baseline,
  int advance,
});

Future<WarpDecisionRecord> _changeLink(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  _LinkChange change,
) async {
  final trigger = journey.resources.origin
      .armBandwidthChangeAfterNextConfirmedChunk({
        change.path,
      }, bandwidthKbps: change.bandwidth);
  final burst = await journey.swipeForward(
    tester,
    count: change.advance,
    afterSequence: journey.focusCursor,
  );
  final profile = await journey.waitForBandwidthTrigger(tester, trigger);
  await journey.waitForRequestProfiles(tester, trigger.requestSequence!, {
    profile.generation - 1,
    profile.generation,
  });
  final decision = await _measureChange(tester, journey, change, profile);
  await _expectMoving(tester, journey, burst.focuses.last);
  return decision;
}

Future<WarpDecisionRecord> _measureChange(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  _LinkChange change,
  ProgressiveOriginLinkProfile profile,
) async {
  final window = await journey.waitForConfirmedLinkWindow(
    tester,
    profile.generation,
    minimumDuration: Duration(
      milliseconds: change.bandwidth == 700 ? 1500 : 500,
    ),
  );
  expect(window.events, isNotEmpty);
  expect(window.achievedBandwidthKbps, lessThanOrEqualTo(change.bandwidth));
  final decision = await journey.waitForDecision(
    tester,
    (decision) =>
        decision.observedAtMs >= window.confirmedAtEpochMs &&
        decision.appliesMeasuredNetworkRate &&
        _rateChanged(decision, change),
    afterSequence: change.baseline.sequence,
  );
  debugPrint(
    'WARP_LINK bandwidth_kbps=${change.bandwidth} '
    'measured_bps=${decision.networkThroughputBps} '
    'planner_Bps=${decision.plannerNetworkRateBytesPerSecond} '
    'confirmed_bytes=${window.bytes} duration_ms=${window.duration.inMilliseconds}',
  );
  return decision;
}

bool _rateChanged(WarpDecisionRecord decision, _LinkChange change) {
  final before = change.baseline.networkThroughputBps;
  return change.bandwidth == 700
      ? decision.networkThroughputBps < before &&
            decision.networkThroughputBps <= 1400000
      : decision.networkThroughputBps > before;
}
