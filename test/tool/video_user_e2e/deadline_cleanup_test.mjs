import assert from "node:assert/strict";
import {access, mkdir, mkdtemp, rm} from "node:fs/promises";
import {tmpdir} from "node:os";
import {join} from "node:path";
import test from "node:test";
import {OwnedLifecycle} from "../../../tool/video_user_e2e/lifecycle.mjs";
import {
  createRunFiles, removeTransientRunFiles,
} from "../../../tool/video_user_e2e/run_files.mjs";
import {withDeadline} from "../../../tool/video_user_e2e/wait.mjs";

test("a hard deadline still tears down children and transient profiles", async () => {
  const root = await mkdtemp(join(tmpdir(), "ghostr-e2e-deadline-"));
  const files = await createRunFiles(root);
  await mkdir(files.profile);
  const killed = [];
  const lifecycle = new OwnedLifecycle({terminate: async (pid) => killed.push(pid)});
  lifecycle.track({pid: 71, label: "server"});
  lifecycle.track({pid: 72, label: "browser"});

  await assert.rejects(runAndClean(files, lifecycle), /total timed out/);

  assert.deepEqual(killed, [72, 71]);
  await assert.rejects(access(files.profile));
  await access(files.artifacts);
  await rm(root, {recursive: true, force: true});
});

async function runAndClean(files, lifecycle) {
  try {
    return await withDeadline({
      run: () => new Promise(() => {}), timeoutMs: 10, label: "total",
    });
  } finally {
    await lifecycle.teardown();
    await removeTransientRunFiles(files);
  }
}
