part of 'warp_evidence_models.dart';

enum WarpReserveCandidateKind { unknown, progressive, hls }

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

WarpReserveCandidateKind _warpCandidateKind(Object? value) {
  return switch (value) {
    'Progressive' => WarpReserveCandidateKind.progressive,
    'Hls' => WarpReserveCandidateKind.hls,
    null => WarpReserveCandidateKind.unknown,
    _ => throw FormatException('Invalid reserve kind: $value'),
  };
}

WarpReserveCandidateState _warpCandidateState(Object? value) {
  return switch (_warpVariantName(value)) {
    'Unprepared' => WarpReserveCandidateState.unprepared,
    'Ready' || 'HlsReady' => WarpReserveCandidateState.ready,
    'Structural' || 'HlsStructural' => WarpReserveCandidateState.structural,
    'InFlight' || 'HlsInFlight' => WarpReserveCandidateState.inFlight,
    'Probing' => WarpReserveCandidateState.probing,
    'Preparing' || 'HlsPending' => WarpReserveCandidateState.preparing,
    'Planned' => WarpReserveCandidateState.planned,
    'Infeasible' => WarpReserveCandidateState.infeasible,
    final state => throw FormatException('Invalid reserve state: $state'),
  };
}
