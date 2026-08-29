part of 'warp_evidence_models.dart';

typedef WarpDecisionPlanPair = ({
  WarpDecisionRecord decision,
  WarpPlanEvidence plan,
});
typedef WarpDecisionPlanPairQuery = ({
  Iterable<WarpDecisionRecord> decisions,
  Iterable<WarpPlanEvidence> plans,
  int afterSequence,
  int afterRevision,
  bool Function(WarpDecisionRecord, WarpPlanEvidence) accepts,
});

WarpDecisionPlanPair? warpFirstDecisionPlanPair(
  WarpDecisionPlanPairQuery query,
) {
  for (final decision in query.decisions) {
    if (decision.sequence <= query.afterSequence) continue;
    for (final plan in query.plans) {
      if (plan.revision <= query.afterRevision) continue;
      if (!plan.sharesPlanningCycleWith(decision)) continue;
      if (query.accepts(decision, plan)) {
        return (decision: decision, plan: plan);
      }
    }
  }
  return null;
}

extension WarpBandwidthPlanEvidence on WarpAllocationPlan {
  int get workBreadth => _work.map((item) => item.postId).toSet().length;

  WarpPlanTransfer? uniqueRetainedActionFor(
    ({int start, int end}) range, {
    required String postId,
  }) {
    WarpPlanTransfer? matched;
    for (final item in retained) {
      if (item.postId != postId ||
          item.actionId == null ||
          item.requestKind != WarpTransferRequestKind.range ||
          item.start != range.start ||
          item.end != range.end) {
        continue;
      }
      if (matched != null) return null;
      matched = item;
    }
    return matched;
  }

  bool retainsActionFrom(WarpAllocationPlan prior, {required int actionId}) {
    final before = prior._retainedAction(actionId);
    final current = _retainedAction(actionId);
    return before != null &&
        current != null &&
        _sameRetainedAction(current, before);
  }

  WarpPlanTransfer? _retainedAction(int actionId) =>
      retained.where((item) => item.actionId == actionId).singleOrNull;

  Iterable<WarpPlanTransfer> get _work sync* {
    yield* allocations;
    yield* retained;
  }
}

extension WarpPlanDecisionCycle on WarpPlanEvidence {
  bool sharesPlanningCycleWith(WarpDecisionRecord decision) {
    final sequence = decisionSequence;
    return sequence != null &&
        sequence == decision.sequence &&
        observedAtMs > 0 &&
        observedAtMs == decision.observedAtMs;
  }
}

extension WarpPlanRetentionCursor on WarpPlanPage {
  int get beforeOldestRetainedRevision {
    return oldestRetainedRevision > 0 ? oldestRetainedRevision - 1 : 0;
  }
}

bool _sameRetainedAction(WarpPlanTransfer left, WarpPlanTransfer right) {
  final action = left.actionId;
  return action != null &&
      right.actionId == action &&
      left.postId == right.postId &&
      left.sourceId == right.sourceId &&
      left.requestKind == right.requestKind &&
      left.start == right.start &&
      left.end == right.end;
}
