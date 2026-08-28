part of 'warp_evidence_models.dart';

enum WarpReserveCandidateState {
  unprepared,
  ready,
  structural,
  inFlight,
  probing,
  preparing,
  planned,
  infeasible,
}

final class WarpReadyReserve {
  const WarpReadyReserve({
    required this.target,
    required this.ready,
    required this.orderedReady,
    required this.structural,
    required this.protected,
    required this.recoveryHorizonMs,
    required this.underflowRiskBps,
    required this.readyCoverageMs,
    required this.candidateCount,
    required this.candidatePostIds,
    this.candidateStates = const [],
  });

  factory WarpReadyReserve.fromJson(Map<String, Object?> json) {
    final candidates = _warpList(json, 'candidates');
    final values = candidates
        .map((item) => _warpObject(item, 'candidate'))
        .toList(growable: false);
    return WarpReadyReserve(
      target: _warpInt(json, 'target'),
      ready: _warpInt(json, 'ready'),
      orderedReady: _warpOrderedReady(json, candidates),
      structural: _warpInt(json, 'structural'),
      protected: _warpInt(json, 'protected'),
      recoveryHorizonMs: _warpInt(json, 'recovery_horizon_ms'),
      underflowRiskBps: _warpInt(json, 'underflow_risk_bps'),
      readyCoverageMs: _warpInt(json, 'ready_coverage_ms'),
      candidateCount: candidates.length,
      candidatePostIds: values
          .map((item) => _warpString(item, 'post'))
          .toList(growable: false),
      candidateStates: values
          .map((item) => _warpCandidateState(_warpRequired(item, 'state')))
          .toList(growable: false),
    );
  }

  final int target;
  final int ready;
  final int orderedReady;
  final int structural;
  final int protected;
  final int recoveryHorizonMs;
  final int underflowRiskBps;
  final int readyCoverageMs;
  final int candidateCount;
  final List<String> candidatePostIds;
  final List<WarpReserveCandidateState> candidateStates;
}

int _warpOrderedReady(Map<String, Object?> json, List<Object?> candidates) {
  final explicit = json['ordered_ready'];
  if (explicit is int) return explicit;
  if (explicit != null) throw const FormatException('Invalid ordered_ready.');
  var ready = 0;
  for (final candidate in candidates) {
    final value = _warpObject(candidate, 'candidate');
    if (_warpVariantName(_warpRequired(value, 'state')) != 'Ready') break;
    ready += 1;
  }
  return ready;
}

WarpReserveCandidateState _warpCandidateState(Object? value) {
  return switch (_warpVariantName(value)) {
    'Unprepared' => WarpReserveCandidateState.unprepared,
    'Ready' => WarpReserveCandidateState.ready,
    'Structural' => WarpReserveCandidateState.structural,
    'InFlight' => WarpReserveCandidateState.inFlight,
    'Probing' => WarpReserveCandidateState.probing,
    'Preparing' => WarpReserveCandidateState.preparing,
    'Planned' => WarpReserveCandidateState.planned,
    'Infeasible' => WarpReserveCandidateState.infeasible,
    final state => throw FormatException('Invalid reserve state: $state'),
  };
}
