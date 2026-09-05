part of 'warp_origin_timeout_fallback_scenario.dart';

final class _OriginTimeoutEvidence {
  _OriginTimeoutEvidence({
    required this.primary,
    required List<ProgressiveOriginRequest> fallback,
    required this.stage,
  }) : fallback = List.unmodifiable(fallback);

  final ProgressiveOriginRequest primary;
  final List<ProgressiveOriginRequest> fallback;
  final WarpFeedPlayerStageEvidence stage;

  Duration get failoverDelay => fallback.first.startedAt - primary.startedAt;
  int get fallbackBytes =>
      fallback.fold(0, (total, request) => total + request.servedBytes);
}

bool warpOriginTimeoutHasBoundedExactFallback(
  Iterable<ProgressiveOriginRequest> requests, {
  required int objectLength,
}) {
  final gets = requests.where((request) => request.method == 'GET').toList();
  // Adaptive dependency closures can split the fallback into more than four
  // requests. Keep a request-storm guard alongside exact once-only coverage.
  if (gets.isEmpty || gets.length > 8) return false;
  if (gets.any((request) => !_isCompletedFallbackBody(request))) return false;
  return ProgressiveOriginCoverage.fromRequests(
    gets,
    objectLength: objectLength,
  ).isExact;
}

bool _isCompletedFallbackBody(ProgressiveOriginRequest request) {
  return request.servedBytes > 0 &&
      request.outcome == ProgressiveOriginRequestOutcome.completed;
}

WarpFeedPlayerStageEvidence? warpOriginTimeoutDecodedStage(
  Iterable<WarpFeedPlayerStageEvidence> attempts,
  PlaybackAssetAuthority readyAuthority,
) {
  for (final stage in attempts.toList().reversed) {
    if (stage.authority != readyAuthority || stage.isTerminal) continue;
    if (stage.firstFrameAt != null) return stage;
  }
  return null;
}
