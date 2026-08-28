part of 'warp_evidence_models.dart';

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

List<WarpPlanTransfer> _warpTransfers(
  Map<String, Object?> json,
  String field,
) => _warpList(json, field)
    .map((item) {
      final value = _warpObject(item, field);
      final request = _warpRequest(_warpChild(value, 'request'));
      return WarpPlanTransfer((
        postId: _warpString(value, 'post'),
        sourceId: _warpString(value, 'source'),
        requestKind: request.kind,
        start: request.start,
        end: request.end,
        reason: _warpString(value, 'reason'),
        actionId: value.containsKey('action_id')
            ? _warpInt(value, 'action_id')
            : null,
        expectedDeliveryMs: _warpInt(
          _warpChild(value, 'utility'),
          'expected_delivery_ms',
        ),
      ));
    })
    .toList(growable: false);

({int start, int end, WarpTransferRequestKind kind}) _warpRequest(
  Map<String, Object?> json,
) {
  if (json['FetchRange'] case final Object? value?) {
    final range = _warpChild(_warpObject(value, 'FetchRange'), 'bytes');
    return (
      start: _warpInt(range, 'start'),
      end: _warpInt(range, 'end'),
      kind: WarpTransferRequestKind.range,
    );
  }
  final whole = _warpChild(json, 'FetchWhole');
  final contract = _warpChild(whole, 'contract');
  final variant = _warpObject(contract.values.single, 'contract');
  final end = variant.values.single;
  if (end is! int) throw const FormatException('Invalid whole request.');
  return (start: 0, end: end, kind: WarpTransferRequestKind.whole);
}

String _warpVariantName(Object? value) {
  if (value is String && value.isNotEmpty) return value;
  final variant = _warpObject(value, 'variant');
  if (variant.length != 1) throw const FormatException('Invalid variant.');
  return variant.keys.single;
}
