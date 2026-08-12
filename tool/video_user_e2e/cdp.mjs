export class CdpConnection {
  #socket;
  #nextId = 1;
  #pending = new Map();
  #listeners = new Map();

  constructor(socket) {
    this.#socket = socket;
    socket.addEventListener("message", (event) => this.#receive(event.data));
    socket.addEventListener("close", () => this.#rejectPending("CDP connection closed"));
    socket.addEventListener("error", () => this.#rejectPending("CDP connection failed"));
  }

  send(method, params = {}, sessionId = undefined) {
    const id = this.#nextId;
    this.#nextId += 1;
    const command = {id, method, params};
    if (sessionId) command.sessionId = sessionId;
    const result = new Promise((resolve, reject) => {
      this.#pending.set(id, {resolve, reject});
    });
    this.#socket.send(JSON.stringify(command));
    return result;
  }

  on(method, listener) {
    const listeners = this.#listeners.get(method) || [];
    listeners.push(listener);
    this.#listeners.set(method, listeners);
  }

  close() {
    this.#socket.close();
  }

  #receive(raw) {
    const message = JSON.parse(raw);
    if (message.id) this.#complete(message);
    else {
      for (const listener of this.#listeners.get(message.method) || []) listener(message);
    }
  }

  #complete(message) {
    const pending = this.#pending.get(message.id);
    if (!pending) return;
    this.#pending.delete(message.id);
    if (message.error) pending.reject(new Error(`CDP ${message.error.message}`));
    else pending.resolve(message.result || {});
  }

  #rejectPending(reason) {
    for (const pending of this.#pending.values()) pending.reject(new Error(reason));
    this.#pending.clear();
  }
}

export async function connectCdp(url, WebSocketClass = globalThis.WebSocket) {
  if (typeof WebSocketClass !== "function") {
    throw new Error("Node.js WebSocket support is required");
  }
  const socket = new WebSocketClass(url);
  await socketOpened(socket);
  return new CdpConnection(socket);
}

function socketOpened(socket) {
  return new Promise((resolve, reject) => {
    socket.addEventListener("open", resolve, {once: true});
    socket.addEventListener(
      "error", () => reject(new Error("cannot connect to CDP")), {once: true},
    );
  });
}
