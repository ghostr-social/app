import assert from "node:assert/strict";
import {mkdtemp, readFile, stat} from "node:fs/promises";
import {tmpdir} from "node:os";
import {join} from "node:path";
import test from "node:test";
import {ArtifactStore} from "../../../tool/video_user_e2e/artifacts.mjs";

test("diagnostics stay valid and inside their byte budgets", async () => {
  const directory = await mkdtemp(join(tmpdir(), "ghostr-e2e-artifacts-"));
  const store = new ArtifactStore({directory, jsonLimit: 256});
  const rows = Array.from({length: 100}, (_, index) => ({
    index, url: `https://media.example/${"x".repeat(40)}`,
  }));

  await store.writeJson("requests.json", rows);

  const path = join(directory, "requests.json");
  assert((await stat(path)).size <= 256);
  assert.equal(JSON.parse(await readFile(path, "utf8")).truncated, true);
  await store.writeText("server.log", "x".repeat(400), 128);
  assert((await stat(join(directory, "server.log"))).size <= 128);
  await store.writeBase64("failure.png", Buffer.alloc(64).toString("base64"), 128);
  await assert.rejects(
    store.writeBase64("oversize.png", Buffer.alloc(129).toString("base64"), 128),
    /byte budget/,
  );
});
