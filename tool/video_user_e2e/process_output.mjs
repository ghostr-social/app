const OUTPUT_LIMIT = 65_536;

export function waitForParsedOutput(input) {
  const drain = new OutputDrain(input.child, input.log);
  return new OutputWaiter(input, drain).wait();
}

class OutputDrain {
  constructor(child, log) {
    this.child = child;
    this.log = log;
    this.output = "";
    this.listeners = new Set();
    this.onData = (chunk) => this.receive(chunk);
    child.stdout?.on("data", this.onData);
    child.stderr?.on("data", this.onData);
    child.once("close", () => this.dispose());
  }

  subscribe(listener) {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  receive(chunk) {
    this.log.append(chunk);
    this.output = `${this.output}${chunk}`.slice(-OUTPUT_LIMIT);
    for (const listener of this.listeners) listener(this.output);
  }

  dispose() {
    this.child.stdout?.off("data", this.onData);
    this.child.stderr?.off("data", this.onData);
    this.listeners.clear();
  }
}

class OutputWaiter {
  constructor(input, drain) {
    this.input = input;
    this.drain = drain;
    this.done = false;
    this.onAbort = () => this.fail(input.signal.reason);
    this.onError = (error) => this.fail(error);
    this.onExit = (code) => this.fail(new Error(`${input.label} exited ${code}`));
  }

  wait() {
    return new Promise((resolve, reject) => {
      this.resolve = resolve;
      this.reject = reject;
      this.unsubscribe = this.drain.subscribe((output) => this.receive(output));
      this.input.child.once("error", this.onError);
      this.input.child.once("exit", this.onExit);
      this.timer = setTimeout(() => {
        this.fail(new Error(`${this.input.label} startup timed out`));
      }, this.input.timeoutMs);
      this.input.signal?.addEventListener("abort", this.onAbort, {once: true});
    });
  }

  receive(output) {
    if (!completeMarkerLine(output, this.input.marker)) return;
    try {
      this.finish(this.input.parse(output));
    } catch (error) {
      this.fail(error);
    }
  }

  finish(value) {
    if (this.done) return;
    this.done = true;
    this.dispose();
    this.resolve(value);
  }

  fail(error) {
    if (this.done) return;
    this.done = true;
    this.dispose();
    this.reject(error || new Error(`${this.input.label} failed`));
  }

  dispose() {
    clearTimeout(this.timer);
    this.unsubscribe?.();
    this.input.child.off("error", this.onError);
    this.input.child.off("exit", this.onExit);
    this.input.signal?.removeEventListener("abort", this.onAbort);
  }
}

function completeMarkerLine(output, marker) {
  const start = output.lastIndexOf(marker);
  return start >= 0 && output.indexOf("\n", start) >= 0;
}
