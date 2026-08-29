import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_evidence_models.dart';

void main() {
  test('plan correlation requires the explicit decision sequence', () {
    final plan = _plan(observedAtMs: 42, decisionSequence: 2);

    expect(plan.sharesPlanningCycleWith(_decision(42, sequence: 2)), isTrue);
    expect(plan.sharesPlanningCycleWith(_decision(42, sequence: 1)), isFalse);
    expect(plan.sharesPlanningCycleWith(_decision(43, sequence: 2)), isFalse);
    expect(
      _plan(
        observedAtMs: 42,
        decisionSequence: null,
      ).sharesPlanningCycleWith(_decision(42, sequence: 2)),
      isFalse,
    );
  });

  test('pair selection skips an earlier decision without a matching plan', () {
    final pair = warpFirstDecisionPlanPair((
      decisions: [_decision(42, sequence: 1), _decision(42, sequence: 2)],
      plans: [_plan(observedAtMs: 42, decisionSequence: 2)],
      afterSequence: 0,
      afterRevision: 0,
      accepts: (_, _) => true,
    ));

    expect(pair?.decision.sequence, 2);
    expect(pair?.plan.decisionSequence, 2);
  });
}

WarpPlanEvidence _plan({
  required int observedAtMs,
  required int? decisionSequence,
}) => WarpPlanEvidence(
  revision: 1,
  decisionSequence: decisionSequence,
  observedAtMs: observedAtMs,
  currentPostId: 'current',
  focusGeneration: 2,
  focusCoversFrom: 1,
  networkStatusGeneration: 1,
  networkClass: WarpNetworkClass.wifi,
  networkProfileGeneration: 1,
  plan: const WarpAllocationPlan(
    mode: 'Safety',
    readyReserve: WarpReadyReserve(
      target: 1,
      ready: 0,
      orderedReady: 0,
      structural: 0,
      protected: 1,
      recoveryHorizonMs: 1000,
      underflowRiskBps: 100,
      readyCoverageMs: 0,
      candidateCount: 1,
      candidatePostIds: [],
    ),
    nextReserveStatus: 'InFlight',
    allocations: [],
    retained: [],
  ),
);

WarpDecisionRecord _decision(int observedAtMs, {required int sequence}) =>
    WarpDecisionRecord(
      sequence: sequence,
      chosenActionId: null,
      outcome: const WarpDecisionOutcome(
        status: 'succeeded',
        bytes: 0,
        elapsedMs: 0,
        failureClass: null,
        claimRefusal: null,
      ),
      selected: null,
      executed: null,
      observedAtMs: observedAtMs,
      networkThroughputBps: 700000,
      plannerNetworkRateBytesPerSecond: 87500,
    );
