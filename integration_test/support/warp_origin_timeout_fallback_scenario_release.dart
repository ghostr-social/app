part of 'warp_origin_timeout_fallback_scenario.dart';

/// How long after the fallback completes the stalled primary may still be
/// open before the engine is judged to have kept it alive.
const _primaryDropGrace = Duration(seconds: 2);

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
    await _expectPrimaryDroppedBeforeRelease(evidence);
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

  /// The engine must drop the stalled primary's connection on its own once
  /// the fallback has delivered the bytes, before the origin ever releases it.
  Future<void> _expectPrimaryDroppedBeforeRelease(
    _OriginTimeoutEvidence evidence,
  ) async {
    try {
      await primaryGate.peerClosed.timeout(_primaryDropGrace);
    } on TimeoutException {
      fail(
        'Stalled primary was not dropped by the client before release; '
        'primary=${evidence.primary.outcome.name}/'
        '${evidence.primary.servedBytes}',
      );
    }
    expect(primaryGate.isPeerClosed, isTrue);
    final closedAt = evidence.primary.peerClosedAt;
    expect(closedAt, isNotNull);
    final fallbackFinishedAt = evidence.fallback.last.finishedAt!;
    expect(closedAt!, lessThan(fallbackFinishedAt));
    debugPrint(
      'WARP_ORIGIN_TIMEOUT peer_closed_ms=${closedAt.inMilliseconds} '
      'fallback_started_ms=${evidence.fallback.first.startedAt.inMilliseconds} '
      'fallback_finished_ms=${fallbackFinishedAt.inMilliseconds}',
    );
  }
}
