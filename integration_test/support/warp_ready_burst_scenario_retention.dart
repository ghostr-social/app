part of 'warp_ready_burst_scenario.dart';

Future<void> runWarpAdaptiveWarmBackScenario(WidgetTester tester) async {
  final journey = await _openFeed(tester);
  await _waitForCached(tester, journey, 0);
  for (var index = 1; index <= 3; index += 1) {
    final cursor = journey.focusCursor;
    await journey.swipeUp(tester);
    final focus = await journey.waitForPublishedFocus(
      tester,
      index,
      afterSequence: cursor,
    );
    await _expectMoving(tester, journey, focus);
    if (index < 3) await _waitForCached(tester, journey, index);
  }
  const replayIds = ['current', 'next', 'third'];
  await journey.waitForNativeStoreCoverage(tester, replayIds);
  final before = await journey.waitForOriginQuiescence(tester, replayIds);
  final reverse = await journey.swipeBackward(
    tester,
    count: 3,
    afterSequence: journey.focusCursor,
    cadence: deviceRapidSwipeCadence,
  );
  await _expectMoving(tester, journey, reverse.focuses.last);
  final after = await journey.waitForOriginQuiescence(tester, replayIds);
  for (final id in replayIds) {
    expect(
      after[id]!.bytes,
      before[id]!.bytes,
      reason: '$id replay uses cached media',
    );
    final coverage = journey.resources.origin.coverageFor(id);
    expect(coverage.isComplete, isTrue, reason: id);
    expect(coverage.duplicateBytes, 0, reason: id);
  }
  debugPrint('WARP_REVERSE origin_before=$before origin_after=$after');
}

Future<void> _waitForCached(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  int index,
) async {
  const ids = ['current', 'next', 'third'];
  final delivery = journey.focus.deliveryForEvent(journey.events[index].id);
  final watch = Stopwatch()..start();
  while (watch.elapsed < const Duration(seconds: 15)) {
    final stored = journey.graph.deliveryProbe.observations
        .where((entry) => entry.snapshot.deliveryId == delivery)
        .lastOrNull
        ?.snapshot;
    if (stored?.bytesPresent ==
            BigInt.from(journey.resources.origin.objectLength) &&
        journey.resources.origin.coverageFor(ids[index]).isComplete) {
      return;
    }
    await journey.pumpFor(tester, const Duration(milliseconds: 25));
  }
  fail(
    'Cache did not complete: ${journey.originRequestEvidence([ids[index]])}',
  );
}
