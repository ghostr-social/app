part of 'warp_evidence_models.dart';

enum WarpNetworkClass { unavailable, wifi, cellular, wired, constrained }

final class WarpPlanEvidence {
  const WarpPlanEvidence({
    required this.revision,
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

final class WarpReadyReserve {
  const WarpReadyReserve({
    required this.target,
    required this.ready,
    required this.structural,
    required this.protected,
    required this.recoveryHorizonMs,
    required this.underflowRiskBps,
    required this.readyCoverageMs,
    required this.candidateCount,
  });

  factory WarpReadyReserve.fromJson(Map<String, Object?> json) =>
      WarpReadyReserve(
        target: _warpInt(json, 'target'),
        ready: _warpInt(json, 'ready'),
        structural: _warpInt(json, 'structural'),
        protected: _warpInt(json, 'protected'),
        recoveryHorizonMs: _warpInt(json, 'recovery_horizon_ms'),
        underflowRiskBps: _warpInt(json, 'underflow_risk_bps'),
        readyCoverageMs: _warpInt(json, 'ready_coverage_ms'),
        candidateCount: _warpList(json, 'candidates').length,
      );

  final int target;
  final int ready;
  final int structural;
  final int protected;
  final int recoveryHorizonMs;
  final int underflowRiskBps;
  final int readyCoverageMs;
  final int candidateCount;
}

final class WarpPlanTransfer {
  const WarpPlanTransfer({
    required this.postId,
    required this.sourceId,
    required this.start,
    required this.end,
    required this.reason,
    required this.actionId,
  });

  final String postId;
  final String sourceId;
  final int start;
  final int end;
  final String reason;
  final int? actionId;
}

List<WarpPlanTransfer> _warpTransfers(
  Map<String, Object?> json,
  String field,
) => _warpList(json, field)
    .map((item) {
      final value = _warpObject(item, field);
      final range = _warpRequestRange(_warpChild(value, 'request'));
      return WarpPlanTransfer(
        postId: _warpString(value, 'post'),
        sourceId: _warpString(value, 'source'),
        start: range.start,
        end: range.end,
        reason: _warpString(value, 'reason'),
        actionId: value.containsKey('action_id')
            ? _warpInt(value, 'action_id')
            : null,
      );
    })
    .toList(growable: false);

({int start, int end}) _warpRequestRange(Map<String, Object?> json) {
  if (json['FetchRange'] case final Object? value?) {
    final range = _warpChild(_warpObject(value, 'FetchRange'), 'bytes');
    return (start: _warpInt(range, 'start'), end: _warpInt(range, 'end'));
  }
  final whole = _warpChild(json, 'FetchWhole');
  final contract = _warpChild(whole, 'contract');
  final variant = _warpObject(contract.values.single, 'contract');
  final end = variant.values.single;
  if (end is! int) throw const FormatException('Invalid whole request.');
  return (start: 0, end: end);
}

String _warpVariantName(Object? value) {
  if (value is String && value.isNotEmpty) return value;
  final variant = _warpObject(value, 'variant');
  if (variant.length != 1) throw const FormatException('Invalid variant.');
  return variant.keys.single;
}

WarpNetworkClass _warpNetworkClass(String value) => switch (value) {
  'Unavailable' => WarpNetworkClass.unavailable,
  'Wifi' => WarpNetworkClass.wifi,
  'Cellular' => WarpNetworkClass.cellular,
  'Wired' => WarpNetworkClass.wired,
  'Constrained' => WarpNetworkClass.constrained,
  _ => throw FormatException('Unknown network class: $value'),
};
