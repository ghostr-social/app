export class OwnedLifecycle {
  #owned = [];
  #terminate;
  #closed = false;

  constructor({terminate = terminateGroup} = {}) {
    this.#terminate = terminate;
  }

  track({pid, label}) {
    if (this.#closed) throw new Error("lifecycle is already closed");
    if (!Number.isSafeInteger(pid) || pid <= 1) {
      throw new Error("safe child PID is required");
    }
    this.#owned.push({pid, label});
  }

  async teardown() {
    if (this.#closed) return;
    this.#closed = true;
    for (const child of this.#owned.reverse()) {
      await this.#terminate(child.pid, "SIGTERM");
    }
    this.#owned = [];
  }
}

async function terminateGroup(pid) {
  signalGroup(pid, "SIGTERM");
  await delay(2_000);
  signalGroup(pid, "SIGKILL");
}

function signalGroup(pid, signal) {
  try {
    process.kill(-pid, signal);
  } catch (error) {
    if (error.code !== "ESRCH") throw error;
  }
}

function delay(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}
