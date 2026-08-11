import assert from "node:assert/strict";
import test from "node:test";
import {
  commitOriginFailure,
  createOriginFailurePlan,
  planOriginFailure,
} from "../../../tool/video_user_e2e/origin_failure_plan.mjs";

test("origin failure planning isolates targeted bodies and preserves every-Nth mode", () => {
  const targeted = createOriginFailurePlan({
    abortFirstAttempts: {video: "v2", count: 2},
  });
  assert.equal(planned(planOriginFailure(targeted, attempt("v2", "HEAD", 1))), false);
  assert.equal(planned(planOriginFailure(targeted, attempt("v0", "GET", 2))), false);
  assert.equal(planned(planOriginFailure(targeted, attempt("v2", "GET", 3))), true);
  const first = planOriginFailure(targeted, attempt("v2", "GET", 4));
  assert.equal(commitOriginFailure(targeted, first), true);
  const second = planOriginFailure(targeted, attempt("v2", "GET", 5));
  assert.equal(commitOriginFailure(targeted, second), true);
  assert.equal(planned(planOriginFailure(targeted, attempt("v2", "GET", 6))), false);

  const generic = createOriginFailurePlan({abortEveryNthRequest: 2});
  assert.equal(planned(planOriginFailure(generic, attempt("v0", "GET", 1))), false);
  const periodic = planOriginFailure(generic, attempt("v7", "GET", 2));
  assert.equal(commitOriginFailure(generic, periodic), true);
});

function attempt(video, method, requestOrdinal) {
  return {video, method, requestOrdinal};
}

function planned(failure) {
  return failure.targeted_failure || failure.periodic_failure;
}
