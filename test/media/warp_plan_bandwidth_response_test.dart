import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_evidence_models.dart';

void main() {
  test('bandwidth response binds one exact retained action', () {
    final baseline = _plan(_retained(7, 100));
    final costly = _plan(_retained(7, 200));
    final replaced = _plan(_retained(8, 200));
    final shifted = _plan(_retained(7, 200, start: 65536));
    final whole = _plan(_retained(7, 200, kind: WarpTransferRequestKind.whole));
    final ambiguous = _planMany([_retained(7, 100), _retained(8, 100)]);
    final collision = _planMany([_retained(7, 100), _otherPost(8, 100)]);

    final sentinel = baseline.uniqueRetainedActionFor((
      start: 0,
      end: 65536,
    ), postId: 'post');
    expect(sentinel?.actionId, 7);
    expect(costly.retainsActionFrom(baseline, actionId: 7), isTrue);
    expect(replaced.retainsActionFrom(baseline, actionId: 7), isFalse);
    expect(shifted.retainsActionFrom(baseline, actionId: 7), isFalse);
    expect(whole.retainsActionFrom(baseline, actionId: 7), isFalse);
    expect(
      ambiguous.uniqueRetainedActionFor((start: 0, end: 65536), postId: 'post'),
      isNull,
    );
    expect(
      collision.uniqueRetainedActionFor((
        start: 0,
        end: 65536,
      ), postId: 'other')?.actionId,
      8,
    );
  });
}

WarpAllocationPlan _plan(WarpPlanTransfer work) => _planMany([work]);

WarpAllocationPlan _planMany(List<WarpPlanTransfer> work) => WarpAllocationPlan(
  mode: 'Safety',
  readyReserve: const WarpReadyReserve(
    target: 2,
    ready: 0,
    orderedReady: 0,
    structural: 0,
    protected: 1,
    recoveryHorizonMs: 1000,
    underflowRiskBps: 100,
    readyCoverageMs: 0,
    candidateCount: 2,
    candidatePostIds: [],
  ),
  nextReserveStatus: 'InFlight',
  allocations: const [],
  retained: work,
);

WarpPlanTransfer _retained(
  int action,
  int cost, {
  int start = 0,
  WarpTransferRequestKind kind = WarpTransferRequestKind.range,
}) => WarpPlanTransfer((
  postId: 'post',
  sourceId: 'origin',
  requestKind: kind,
  start: start,
  end: start + 65536,
  reason: 'NextStartability',
  actionId: action,
  expectedDeliveryMs: cost,
));

WarpPlanTransfer _otherPost(int action, int cost) => WarpPlanTransfer((
  postId: 'other',
  sourceId: 'origin',
  requestKind: WarpTransferRequestKind.range,
  start: 0,
  end: 65536,
  reason: 'NextStartability',
  actionId: action,
  expectedDeliveryMs: cost,
));
