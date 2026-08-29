part of 'warp_evidence_models.dart';

final class WarpDecisionRecord {
  const WarpDecisionRecord({
    required this.sequence,
    required this.chosenActionId,
    required this.outcome,
    required this.selected,
    required this.executed,
    required this.observedAtMs,
    required this.networkThroughputBps,
    required this.plannerNetworkRateBytesPerSecond,
    this.hasWarpDecision = false,
    this.additionalRequestSlotDemanded = false,
  });

  factory WarpDecisionRecord.fromJson(Map<String, Object?> json) {
    final replay = _warpDecisionReplay(json);
    return WarpDecisionRecord(
      sequence: _warpInt(json, 'sequence'),
      chosenActionId: _warpOptionalInt(json, 'chosen_action_id'),
      outcome: WarpDecisionOutcome.fromJson(
        _warpChild(json, 'eventual_outcome'),
      ),
      selected: _warpSelected(json),
      executed: _warpExecuted(json),
      observedAtMs: replay.observedAtMs,
      networkThroughputBps: replay.throughputBps,
      plannerNetworkRateBytesPerSecond: _warpPlannerNetworkRate(json),
      hasWarpDecision: json['warp_decision'] != null,
      additionalRequestSlotDemanded: _warpAdditionalSlotDemand(json),
    );
  }

  final int sequence;
  final int? chosenActionId;
  final WarpDecisionOutcome outcome;
  final WarpDecisionAction? selected;
  final WarpExecutedRequest? executed;
  final int observedAtMs;
  final int networkThroughputBps;
  final int? plannerNetworkRateBytesPerSecond;
  final bool hasWarpDecision;
  final bool additionalRequestSlotDemanded;
}

typedef _WarpDecisionReplay = ({int observedAtMs, int throughputBps});

_WarpDecisionReplay _warpDecisionReplay(Map<String, Object?> json) {
  final raw = json['replay_state'];
  if (raw == null) return (observedAtMs: 0, throughputBps: 0);
  final replay = _warpObject(raw, 'replay_state');
  final network = _warpChild(replay, 'network');
  return (
    observedAtMs: _warpInt(replay, 'observed_at_ms'),
    throughputBps: _warpInt(network, 'throughput_bps'),
  );
}

bool _warpAdditionalSlotDemand(Map<String, Object?> json) {
  final raw = json['warp_decision'];
  if (raw == null) return false;
  return _warpBool(
    _warpObject(raw, 'warp_decision'),
    'additional_request_slot_demanded',
  );
}
