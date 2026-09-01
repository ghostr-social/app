import 'warp_evidence_models.dart';

const _maximumPlanEvidence = 6;
const _maximumIdLength = 8;

String formatWarpFocusPlanTimeoutEvidence(
  Iterable<WarpPlanEvidence> plans, {
  required BigInt? focusGeneration,
}) {
  final causal = <WarpPlanEvidence>[];
  if (focusGeneration != null) {
    causal.addAll(
      plans.where((plan) => plan.coversFocusGeneration(focusGeneration)),
    );
    causal.sort((left, right) => right.revision.compareTo(left.revision));
  }
  final target = causal.isEmpty ? null : causal.first.currentPostId;
  final summaries = causal
      .take(_maximumPlanEvidence)
      .map((plan) => _planSummary(plan, target));
  return 'focus=${focusGeneration ?? 'missing'} '
      'target=${_boundedId(target)} '
      'plans=${summaries.isEmpty ? 'none' : summaries.join('|')}';
}

String _planSummary(WarpPlanEvidence plan, String? target) {
  final allocation = _workSummary(plan.plan.allocations, target);
  final retained = _workSummary(plan.plan.retained, target);
  return '${plan.revision}['
      'g=${plan.focusCoversFrom}-${plan.focusGeneration},'
      'c=${_boundedId(plan.currentPostId)},a=$allocation,r=$retained]';
}

String _workSummary(Iterable<WarpPlanTransfer> work, String? target) {
  final matches = work.where((item) => item.postId == target).map((item) {
    final action = item.actionId?.toString() ?? 'new';
    return '${item.start}-${item.end}#$action:${item.reason}';
  });
  return matches.isEmpty ? 'none' : matches.join('&');
}

String _boundedId(String? value) {
  if (value == null) return 'missing';
  if (value.length <= _maximumIdLength) return value;
  return value.substring(0, _maximumIdLength);
}
