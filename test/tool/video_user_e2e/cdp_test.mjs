import assert from "node:assert/strict";
import test from "node:test";
import {CdpConnection} from "../../../tool/video_user_e2e/cdp.mjs";

class FakeSocket {
  listeners = new Map();
  sent = [];
  addEventListener(name, listener) { this.listeners.set(name, listener); }
  send(raw) {
    const command = JSON.parse(raw);
    this.sent.push(command);
    queueMicrotask(() => this.emit("message", {data: JSON.stringify({
      id: command.id, result: {echo: command.method},
    })}));
  }
  emit(name, event) { this.listeners.get(name)?.(event); }
  close() {}
}

test("CDP correlates commands and streams page-session events", async () => {
  const socket = new FakeSocket();
  const cdp = new CdpConnection(socket);
  const events = [];
  cdp.on("Network.requestWillBeSent", (event) => events.push(event.params.request.url));

  const result = await cdp.send("Network.enable", {}, "page-session");
  socket.emit("message", {data: JSON.stringify({
    method: "Network.requestWillBeSent", sessionId: "page-session",
    params: {request: {url: "http://127.0.0.1/video"}},
  })});

  assert.deepEqual(result, {echo: "Network.enable"});
  assert.equal(socket.sent[0].sessionId, "page-session");
  assert.deepEqual(events, ["http://127.0.0.1/video"]);
});
