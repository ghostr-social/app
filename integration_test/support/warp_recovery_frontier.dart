import 'warp_evidence_models.dart';

typedef WarpRecoveryFrontierQuery = ({
  List<String> futureRosterPaths,
  List<String> candidatePaths,
  List<String> projectedPaths,
  List<WarpReserveCandidateState> candidateStates,
  int orderedReady,
  int candidateCount,
  int minimumReadyDepth,
});

typedef WarpRecoveryFrontier = ({
  int readyDepth,
  String firstUnreadyPath,
  Set<String> transitionPaths,
});

WarpRecoveryFrontier warpRecoveryFrontier(WarpRecoveryFrontierQuery query) {
  final depth = query.orderedReady;
  _validateRecoveryDepth(query);
  _validateRecoveryRoster(query);
  _validateRecoveryPrefix(query);
  final firstUnready = query.candidatePaths[depth];
  final transition = _nonReadyCandidatePaths(query);
  if (!transition.contains(firstUnready)) {
    throw StateError('The ordered frontier is already player-ready.');
  }
  return (
    readyDepth: depth,
    firstUnreadyPath: firstUnready,
    transitionPaths: transition,
  );
}

void _validateRecoveryDepth(WarpRecoveryFrontierQuery query) {
  if (query.orderedReady < query.minimumReadyDepth ||
      query.orderedReady >= query.candidateCount) {
    throw StateError('The plan has no certified recovery frontier.');
  }
}

void _validateRecoveryRoster(WarpRecoveryFrontierQuery query) {
  if (query.candidateCount > query.futureRosterPaths.length ||
      query.candidatePaths.length != query.candidateCount ||
      query.candidateStates.length != query.candidateCount ||
      query.projectedPaths.length < query.orderedReady) {
    throw StateError('The recovery evidence does not cover the roster.');
  }
  final roster = query.futureRosterPaths.take(query.candidateCount).toList();
  if (!_sameValues(query.candidatePaths, roster)) {
    throw StateError('The plan candidates do not match the future roster.');
  }
}

void _validateRecoveryPrefix(WarpRecoveryFrontierQuery query) {
  final depth = query.orderedReady;
  for (var index = 0; index < depth; index += 1) {
    final expected = query.candidatePaths[index];
    if (query.projectedPaths[index] != expected ||
        query.candidateStates[index] != WarpReserveCandidateState.ready) {
      throw StateError('Projected readiness is not the ordered roster prefix.');
    }
  }
  if (query.candidateStates[depth] == WarpReserveCandidateState.ready ||
      !_isOrderedSubset(query.projectedPaths, query.candidatePaths)) {
    throw StateError('Projected readiness is outside the modeled roster.');
  }
}

Set<String> _nonReadyCandidatePaths(WarpRecoveryFrontierQuery query) {
  return {
    for (
      var index = query.orderedReady;
      index < query.candidateCount;
      index += 1
    )
      if (query.candidateStates[index] != WarpReserveCandidateState.ready)
        query.candidatePaths[index],
  };
}

bool _sameValues(List<String> left, List<String> right) {
  if (left.length != right.length) return false;
  return Iterable<int>.generate(
    left.length,
  ).every((index) => left[index] == right[index]);
}

bool _isOrderedSubset(List<String> values, List<String> ordered) {
  var cursor = 0;
  for (final value in values) {
    while (cursor < ordered.length && ordered[cursor] != value) {
      cursor += 1;
    }
    if (cursor == ordered.length) return false;
    cursor += 1;
  }
  return true;
}
