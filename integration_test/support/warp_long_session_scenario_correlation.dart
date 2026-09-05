part of 'warp_long_session_scenario.dart';

const _cancellationPostIndex = _longSessionCancellationWarmup + 1;

typedef _CancellationDecisionBinding = ({
  int actionId,
  String postId,
  int start,
  int end,
});

extension _WarpLongSessionCorrelation on _WarpLongSessionDriver {
  Future<_CancellationDecisionBinding> _cancellationDecisionBinding() async {
    final focus = graph.focus.occurrences.last;
    final generation = graph.focus.generationFor(focus);
    if (generation == null) throw StateError('Focus generation is absent.');
    final plan = await _latestPlanCovering(generation);
    if (plan == null) throw StateError('No causal plan is retained.');
    final postId = _candidatePostId(plan);
    final range = cancellationRequest.range;
    if (range == null) throw StateError('Held request is not ranged.');
    final transfer = plan.plan.uniqueRetainedActionFor(range, postId: postId);
    final actionId = transfer?.actionId;
    if (actionId == null) throw StateError('Held action is not retained.');
    return (
      actionId: actionId,
      postId: postId,
      start: range.start,
      end: range.end,
    );
  }

  Future<WarpPlanEvidence?> _latestPlanCovering(BigInt generation) async {
    final overview = await graph.evidence.page(limit: 1);
    final page = await graph.evidence.page(
      afterRevision: overview.planPage.beforeLatestRetainedRevision,
      limit: 1,
    );
    for (final plan in page.planPage.records.reversed) {
      if (plan.coversFocusGeneration(generation)) return plan;
    }
    return null;
  }

  String _candidatePostId(WarpPlanEvidence plan) {
    final state = graph.cubit.state as FeedLoaded;
    final target = scenario.events[_cancellationPostIndex].id;
    final targetIndex = state.posts.indexWhere(
      (post) => post.id.value == target,
    );
    final slot = targetIndex - state.activeIndex - 1;
    final candidates = plan.plan.readyReserve.candidatePostIds;
    if (slot < 0 || slot >= candidates.length) {
      throw StateError('Cancellation candidate slot $slot is absent.');
    }
    return candidates[slot];
  }
}
