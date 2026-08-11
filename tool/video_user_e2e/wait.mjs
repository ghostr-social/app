export async function poll(input) {
  const started = Date.now();
  let last;
  while (Date.now() - started < input.timeoutMs) {
    requireActive(input.signal);
    last = await input.read();
    if (input.accept(last)) return last;
    await delay(input.intervalMs ?? 250, input.signal);
  }
  throw new Error(`${input.label} timed out after ${input.timeoutMs} ms: ${describe(last)}`);
}

export async function withDeadline({run, timeoutMs, label}) {
  const controller = new AbortController();
  const operation = Promise.resolve().then(() => run(controller.signal));
  operation.catch(() => {});
  let timer;
  const deadline = new Promise((_, reject) => {
    timer = setTimeout(() => {
      const error = new Error(`${label} timed out`);
      controller.abort(error);
      reject(error);
    }, timeoutMs);
  });
  try {
    return await Promise.race([operation, deadline]);
  } finally {
    clearTimeout(timer);
  }
}

export function delay(milliseconds, signal) {
  requireActive(signal);
  return new Promise((resolve, reject) => {
    const finish = () => {
      signal?.removeEventListener("abort", abort);
      resolve();
    };
    const abort = () => {
      clearTimeout(timer);
      signal?.removeEventListener("abort", abort);
      reject(signal.reason || new Error("operation aborted"));
    };
    const timer = setTimeout(finish, milliseconds);
    signal?.addEventListener("abort", abort, {once: true});
  });
}

function requireActive(signal) {
  if (signal?.aborted) throw signal.reason || new Error("operation aborted");
}

function describe(value) {
  if (value == null) return "no sample";
  try {
    return JSON.stringify(value).slice(0, 240);
  } catch {
    return String(value).slice(0, 240);
  }
}
