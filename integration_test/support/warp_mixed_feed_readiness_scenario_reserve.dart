part of 'warp_mixed_feed_readiness_scenario.dart';

Future<void> _waitForHeldThird(
  WidgetTester tester,
  WarpMixedFeedRuntime runtime,
  ProgressiveOriginPreBodyGate gate,
) {
  return _waitUntil(tester, runtime, () => gate.isReached);
}

Future<WarpPlanEvidence> _waitForCanonicalHlsReserve(
  WidgetTester tester,
  WarpMixedFeedRuntime runtime,
  PlaybackFocus startup,
  ProgressiveOriginPreBodyGate gate,
) async {
  final generation = runtime.graph.focus.generationFor(startup);
  expect(generation, isNotNull, reason: _evidence(runtime));
  var cursor = 0;
  final watch = Stopwatch()..start();
  while (watch.elapsed < const Duration(seconds: 20)) {
    final page = await runtime.graph.evidence.page(afterRevision: cursor);
    final match = _matchingHlsReserve(page, generation!);
    if (match != null) {
      _expectHeldThird(runtime, gate);
      return _reportHlsReserve(runtime, match);
    }
    if (page.planPage.records.isNotEmpty) {
      cursor = page.planPage.records.last.revision;
    }
    await tester.pump(const Duration(milliseconds: 50));
    await Future<void>.delayed(const Duration(milliseconds: 20));
  }
  fail('${_evidence(runtime)} No canonical decoder-ready HLS reserve.');
}

void _expectHeldThird(
  WarpMixedFeedRuntime runtime,
  ProgressiveOriginPreBodyGate gate,
) {
  expect(gate.isReached, isTrue, reason: _evidence(runtime));
  expect(gate.isReleased, isFalse, reason: _evidence(runtime));
  expect(gate.timedOut, isFalse, reason: _evidence(runtime));
  expect(_isPlayerReady(runtime, 2), isFalse, reason: _evidence(runtime));
}

WarpPlanEvidence? _matchingHlsReserve(
  WarpEvidencePage page,
  BigInt generation,
) {
  for (final record in page.planPage.records) {
    final reserve = record.plan.readyReserve;
    if (record.coversFocusGeneration(generation) &&
        reserve.candidateKinds.firstOrNull == WarpReserveCandidateKind.hls &&
        reserve.candidateStates.firstOrNull ==
            WarpReserveCandidateState.ready &&
        reserve.orderedReady >= 1) {
      return record;
    }
  }
  return null;
}

WarpPlanEvidence _reportHlsReserve(
  WarpMixedFeedRuntime runtime,
  WarpPlanEvidence record,
) {
  final reserve = record.plan.readyReserve;
  debugPrint(
    'WARP_HLS_RESERVE revision=${record.revision} mode=${record.plan.mode} '
    'target=${reserve.target} ordered=${reserve.orderedReady} '
    'ready=${reserve.ready} kinds=${reserve.candidateKinds.join("|")} '
    'candidates=${reserve.candidatePostIds.join("|")}',
  );
  expect(reserve.ready, greaterThanOrEqualTo(1));
  return record;
}
