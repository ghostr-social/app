import {executeTimedActions} from "./impairment_executor.mjs";

export async function runImpairmentJourney(input) {
  if (input.scenario === "protected_transitions") return runProtectedTransitions(input);
  if (input.actions.some((action) => action.kind === "focus")) {
    return runRapidFocus(input);
  }
  return runStandardFocus(input);
}

async function runProtectedTransitions(input) {
  const lastIndex = 3;
  for (let index = 0; index <= lastIndex; index += 1) {
    const id = input.ids[index];
    const at_ms = input.now() - input.startedAt;
    await input.click(id);
    const transitionOnly = index < lastIndex;
    input.trace.clicks.push(protectedClick(id, at_ms, transitionOnly));
    await (transitionOnly ? input.watchStart : input.watch)(id);
  }
}

function protectedClick(id, at_ms, transitionOnly) {
  return {
    id,
    at_ms,
    protected_transition: true,
    ...(transitionOnly ? {transition_only: true} : {}),
  };
}

async function runRapidFocus(input) {
  await executeTimedActions({
    actions: input.actions,
    startedAt: input.startedAt,
    clock: input.now,
    wait: input.wait,
    signal: input.signal,
    focus: (action) => performFocus(input, action),
  });
  const final = input.actions.at(-1);
  await input.watch(input.ids[final.payload.index]);
}

async function runStandardFocus(input) {
  const immediate = input.actions.filter((action) => action.at_ms === 0);
  const scheduled = input.actions.filter((action) => action.at_ms > 0);
  await runControls(input, immediate);
  await Promise.all([runDefaultClicks(input), runControls(input, scheduled)]);
}

async function runControls(input, actions) {
  await executeTimedActions({
    actions,
    startedAt: input.startedAt,
    clock: input.now,
    wait: input.wait,
    signal: input.signal,
    send: input.send,
  });
}

async function runDefaultClicks(input) {
  const sequence = [0, 1, 2, 3];
  for (const index of sequence) {
    const id = input.ids[index];
    const at_ms = input.now() - input.startedAt;
    await input.click(id);
    input.trace.clicks.push({id, at_ms});
    await input.watch(id);
  }
}

async function performFocus(input, action) {
  const id = input.ids[action.payload.index];
  const at_ms = input.now() - input.startedAt;
  await input.click(id);
  input.trace.clicks.push({
    id,
    at_ms,
    superseded: action.payload.superseded,
  });
}
