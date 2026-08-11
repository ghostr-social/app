import {IMPAIRMENT_SCENARIOS} from "./impairment_scenarios.mjs";

const CHECK = "--check-prerequisites";
const SCENARIO = "--scenario=";

export function parseVideoE2eArguments(arguments_) {
  const unknown = arguments_.filter((value) => value !== CHECK && !value.startsWith(SCENARIO));
  if (unknown.length) throw new Error(`unknown video-user-e2e argument: ${unknown.join(", ")}`);
  const scenarios = arguments_
    .filter((value) => value.startsWith(SCENARIO))
    .map((value) => value.slice(SCENARIO.length));
  if (scenarios.length > 1) throw new Error("video-user-e2e accepts only one scenario");
  const scenario = scenarios[0] ?? null;
  if (scenario && !(scenario in IMPAIRMENT_SCENARIOS)) {
    throw new Error(`unknown scenario: ${scenario}`);
  }
  return {checkOnly: arguments_.includes(CHECK), scenario};
}
