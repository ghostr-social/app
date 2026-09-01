part of 'warp_origin_timeout_fallback_scenario.dart';

extension _OriginTimeoutFallbackRelease on _OriginTimeoutFallbackScenario {
  Future<void> expectTransientPrimaryFailure(WidgetTester tester) async {
    final pair = await journey.waitForDecisionPlanPair(tester, (item, plan) {
      final selected = item.selected;
      final executed = item.executed;
      final candidates = plan.plan.readyReserve.candidatePostIds;
      return item.outcome.failureClass == 'Transient' &&
          candidates.isNotEmpty &&
          selected?.postId == candidates.first &&
          executed?.postId == selected?.postId &&
          executed?.sourceId == selected?.sourceId;
    }, afterSequence: decisionBaseline);
    expect(pair.decision.outcome.bytes, anyOf(isNull, 0));
  }

  Future<void> releaseLatePrimary(
    WidgetTester tester,
    _OriginTimeoutEvidence evidence,
  ) async {
    primaryGate.release();
    final watch = Stopwatch()..start();
    while (evidence.primary.outcome ==
            ProgressiveOriginRequestOutcome.serving &&
        watch.elapsed < const Duration(seconds: 5)) {
      await journey.pumpFor(tester, const Duration(milliseconds: 50));
    }
    expect(
      evidence.primary.outcome,
      ProgressiveOriginRequestOutcome.clientCanceled,
    );
    expect(evidence.primary.servedBytes, 0);
    final sequence = journey.resources.origin.requestSequenceFor(
      evidence.primary,
    );
    expect(
      journey.resources.origin.activeIncompleteRequestSequences,
      isNot(contains(sequence)),
    );
    _expectOnePrimaryGet();
    _expectBoundedExactFallback();
  }
}
