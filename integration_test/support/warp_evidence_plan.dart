part of 'warp_evidence_models.dart';

final class WarpPlanEvidence {
  const WarpPlanEvidence({
    required this.revision,
    required this.decisionSequence,
    required this.observedAtMs,
    required this.currentPostId,
    required this.focusGeneration,
    required this.focusCoversFrom,
    required this.networkStatusGeneration,
    required this.networkClass,
    required this.networkProfileGeneration,
    required this.plan,
  });

  factory WarpPlanEvidence.fromJson(Map<String, Object?> json) =>
      WarpPlanEvidence(
        revision: _warpInt(json, 'revision'),
        decisionSequence: json.containsKey('decision_sequence')
            ? _warpOptionalInt(json, 'decision_sequence')
            : null,
        observedAtMs: _warpInt(json, 'observed_at_ms'),
        currentPostId: _warpOptionalString(json, 'current_post_id'),
        focusGeneration: _warpOptionalInt(json, 'focus_generation'),
        focusCoversFrom: _warpOptionalInt(json, 'focus_covers_from'),
        networkStatusGeneration: _warpInt(json, 'network_status_generation'),
        networkClass: _warpNetworkClass(_warpString(json, 'network_class')),
        networkProfileGeneration: _warpInt(json, 'network_profile_generation'),
        plan: WarpAllocationPlan.fromJson(_warpChild(json, 'plan')),
      );

  final int revision;
  final int? decisionSequence;
  final int observedAtMs;
  final String? currentPostId;
  final int? focusGeneration;
  final int? focusCoversFrom;
  final int networkStatusGeneration;
  final WarpNetworkClass networkClass;
  final int networkProfileGeneration;
  final WarpAllocationPlan plan;

  bool coversFocusGeneration(BigInt generation) {
    final first = focusCoversFrom;
    final last = focusGeneration;
    if (first == null || last == null) return false;
    if (first <= 0 || last < first) return false;
    final value = generation.toInt();
    return first <= value && value <= last;
  }
}

final class WarpAllocationPlan {
  const WarpAllocationPlan({
    required this.mode,
    required this.readyReserve,
    required this.nextReserveStatus,
    required this.allocations,
    required this.retained,
  });

  factory WarpAllocationPlan.fromJson(Map<String, Object?> json) {
    _warpList(json, 'evictions');
    return WarpAllocationPlan(
      mode: _warpString(json, 'mode'),
      readyReserve: WarpReadyReserve.fromJson(
        _warpChild(json, 'ready_reserve'),
      ),
      nextReserveStatus: _warpVariantName(_warpRequired(json, 'next_reserve')),
      allocations: _warpTransfers(json, 'allocations'),
      retained: _warpTransfers(json, 'retained'),
    );
  }

  final String mode;
  final WarpReadyReserve readyReserve;
  final String nextReserveStatus;
  final List<WarpPlanTransfer> allocations;
  final List<WarpPlanTransfer> retained;
}
