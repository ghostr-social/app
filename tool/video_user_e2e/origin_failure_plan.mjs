export function createOriginFailurePlan(options = {}) {
  const target = options.abortFirstAttempts;
  return {
    everyNthRequest: options.abortEveryNthRequest ?? 0,
    targetVideo: target?.video ?? null,
    remainingTargetAttempts: target?.count ?? 0,
  };
}

export function planOriginFailure(plan, attempt) {
  return {
    targeted_failure: isTargetBody(plan, attempt),
    periodic_failure: plan.everyNthRequest > 0
      && attempt.requestOrdinal % plan.everyNthRequest === 0,
  };
}

export function commitOriginFailure(plan, failure) {
  if (failure.targeted_failure && plan.remainingTargetAttempts > 0) {
    plan.remainingTargetAttempts -= 1;
    return true;
  }
  return failure.periodic_failure;
}

function isTargetBody(plan, attempt) {
  return attempt.method !== "HEAD"
    && attempt.video === plan.targetVideo
    && plan.remainingTargetAttempts > 0;
}
