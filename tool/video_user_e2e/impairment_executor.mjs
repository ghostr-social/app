import {delay} from "./wait.mjs";

export async function executeTimedActions(input) {
  for (const action of input.actions) {
    const due = input.startedAt + action.at_ms - (input.clock ?? Date.now)();
    await (input.wait ?? delay)(Math.max(0, due), input.signal);
    const execute = action.kind === "focus" ? input.focus : input.send;
    if (!execute) throw new Error(`no executor for ${action.kind} impairment`);
    await execute(action);
  }
}

export async function sendControlAction(server, action, request = fetch) {
  const response = await request(`${server}${controlPath(action.kind)}`, {
    method: "PUT",
    headers: {"content-type": "application/json"},
    body: JSON.stringify(action.payload),
  });
  if (!response.ok) {
    throw new Error(`${action.kind} impairment failed: HTTP ${response.status}`);
  }
}

function controlPath(kind) {
  if (kind === "network") return "/api/network";
  if (kind === "storage") return "/api/storage";
  throw new Error(`unsupported impairment action: ${kind}`);
}
