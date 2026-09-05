part of 'warp_feed_playback_journey.dart';

extension _WarpPairDiagnostics on _WarpPairWait {
  Future<void> _reportPairDiagnostics() async {
    final decisions = await journey.sampleDecisions();
    for (final decision in decisions) {
      if (decision.sequence <= query.afterSequence) continue;
      final plans = _plans
          .where((plan) {
            return plan.decisionSequence == decision.sequence ||
                plan.observedAtMs == decision.observedAtMs;
          })
          .toList(growable: false);
      final accepted = plans.any((plan) {
        return plan.sharesPlanningCycleWith(decision) &&
            query.accepts(decision, plan);
      });
      debugPrint(
        'WARP_PAIR sequence=${decision.sequence} at=${decision.observedAtMs} '
        'throughput_bps=${decision.networkThroughputBps} '
        'planner_Bps=${decision.plannerNetworkRateBytesPerSecond} '
        'plans=${plans.map(_pairPlanSummary).join('|')} accepted=$accepted',
      );
    }
  }
}

String _pairPlanSummary(WarpPlanEvidence plan) =>
    '${plan.revision}/${plan.decisionSequence}@${plan.observedAtMs}:'
    '${plan.networkStatusGeneration}:'
    '${plan.focusCoversFrom}-${plan.focusGeneration}:${plan.plan.workBreadth}';
