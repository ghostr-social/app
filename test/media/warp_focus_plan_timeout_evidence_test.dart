import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_evidence_models.dart';
import '../../integration_test/support/warp_focus_plan_timeout_evidence.dart';

void main() {
  test('reports only latest causal target work with bounded private ids', () {
    final plans = [
      _plan(10, 5, allocations: [_work('target-private', 0, 10)]),
      _plan(11, 7, retained: [_work('target-private', 0, 43)]),
      _plan(
        12,
        7,
        allocations: [
          _work('other-private', 0, null),
          _work('target-private', 65536, null),
        ],
      ),
      _plan(13, 8, retained: [_work('target-private', 10, 44)]),
    ];

    final evidence = formatWarpFocusPlanTimeoutEvidence(
      plans,
      focusGeneration: BigInt.from(7),
    );

    expect(
      evidence,
      'focus=7 target=target-p '
      'plans=12[g=7-7,c=target-p,a=65536-131072#new:'
      'CurrentStartability,r=none]|'
      '11[g=7-7,c=target-p,a=none,r=0-65536#43:CurrentStartability]',
    );
    expect(evidence, isNot(contains('private')));
    expect(evidence, isNot(contains('10[')));
    expect(evidence, isNot(contains('13[')));
  });
}

WarpPlanEvidence _plan(
  int revision,
  int generation, {
  List<WarpPlanTransfer> allocations = const [],
  List<WarpPlanTransfer> retained = const [],
}) => WarpPlanEvidence(
  revision: revision,
  decisionSequence: revision,
  observedAtMs: revision,
  currentPostId: 'target-private',
  focusGeneration: generation,
  focusCoversFrom: generation,
  networkStatusGeneration: 1,
  networkClass: WarpNetworkClass.wifi,
  networkProfileGeneration: 1,
  plan: WarpAllocationPlan(
    mode: 'Emergency',
    readyReserve: _reserve,
    nextReserveStatus: 'Granted',
    allocations: allocations,
    retained: retained,
  ),
);

WarpPlanTransfer _work(String post, int start, int? action) =>
    WarpPlanTransfer((
      postId: post,
      sourceId: 'source-private',
      requestKind: WarpTransferRequestKind.range,
      start: start,
      end: start + 65536,
      reason: 'CurrentStartability',
      actionId: action,
      expectedDeliveryMs: 1,
    ));

const _reserve = WarpReadyReserve(
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
);
