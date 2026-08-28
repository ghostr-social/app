part of 'warp_evidence_models.dart';

int? _warpPlannerNetworkRate(Map<String, Object?> json) {
  final decision = _warpOptionalObject(json, 'warp_decision');
  final capsule = _warpOptionalObject(decision, 'planner_replay_capsule');
  final context = _warpOptionalObject(capsule, 'context');
  if (context == null) return null;
  return _warpInt(
    _warpChild(context, 'limits'),
    'network_rate_bytes_per_second',
  );
}

Map<String, Object?>? _warpOptionalObject(
  Map<String, Object?>? json,
  String field,
) {
  final value = json?[field];
  return value == null ? null : _warpObject(value, field);
}

extension WarpPlannerNetworkEvidence on WarpDecisionRecord {
  bool get appliesMeasuredNetworkRate {
    final rate = plannerNetworkRateBytesPerSecond;
    final bytesPerSecond = networkThroughputBps ~/ 8;
    final expected = bytesPerSecond < 1 ? 1 : bytesPerSecond;
    return rate == expected;
  }
}
