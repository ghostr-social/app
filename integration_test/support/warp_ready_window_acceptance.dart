enum WarpReadyWindowGoal { consumeBurst, recoveryFrontier, fillTarget }

typedef WarpEvidenceCursor = ({int revision, int sequence});
typedef WarpReadyRevision = ({int preparation, int plan});
typedef WarpReadySequence = ({int observation, int after});
typedef WarpReadyThreshold = ({
  int contiguous,
  int ordered,
  int minimum,
  int target,
  int candidateCount,
  WarpReadyWindowGoal goal,
});

int? warpNewestCausalEvidenceIndex({
  required List<WarpEvidenceCursor> history,
  required int afterRevision,
  required int afterSequence,
}) {
  for (var index = history.length - 1; index >= 0; index -= 1) {
    final cursor = history[index];
    if (cursor.revision <= afterRevision) continue;
    if (cursor.sequence <= afterSequence) continue;
    return index;
  }
  return null;
}

bool warpReadyWindowAccepted({
  required int contiguousDepth,
  required int minimumDepth,
  required int planTarget,
  required WarpReadyWindowGoal goal,
}) {
  if (contiguousDepth < minimumDepth) return false;
  if (goal != WarpReadyWindowGoal.fillTarget) return true;
  return contiguousDepth >= planTarget;
}

bool warpReadyEvidenceAccepted({
  required WarpReadyRevision revision,
  required WarpReadySequence sequence,
  required WarpReadyThreshold readiness,
}) {
  if (revision.preparation != revision.plan) return false;
  if (sequence.observation <= sequence.after) return false;
  if (readiness.goal == WarpReadyWindowGoal.recoveryFrontier &&
      readiness.ordered >= readiness.candidateCount) {
    return false;
  }
  final certifiedDepth = readiness.contiguous < readiness.ordered
      ? readiness.contiguous
      : readiness.ordered;
  return warpReadyWindowAccepted(
    contiguousDepth: certifiedDepth,
    minimumDepth: readiness.minimum,
    planTarget: readiness.target,
    goal: readiness.goal,
  );
}
