import assert from "node:assert/strict";
import {mkdtemp, mkdir, writeFile, access, rm} from "node:fs/promises";
import {join} from "node:path";
import {tmpdir} from "node:os";
import test from "node:test";
import {
  createRunFiles, removeTransientRunFiles,
} from "../../../tool/video_user_e2e/run_files.mjs";

test("cleanup removes transient state and preserves retained artifacts", async () => {
  const root = await mkdtemp(join(tmpdir(), "ghostr-e2e-files-"));
  await mkdir(join(root, "rust", "target"), {recursive: true});
  const files = await createRunFiles(root);
  await mkdir(files.profile, {recursive: true});
  await writeFile(join(files.profile, "temporary"), "x");
  await writeFile(join(files.artifacts, "result.json"), "{}");

  await removeTransientRunFiles(files);

  await assert.rejects(access(files.profile));
  await access(join(files.artifacts, "result.json"));
  await rm(root, {recursive: true, force: true});
});

test("retained artifacts cannot live under Cargo target", async () => {
  const root = await mkdtemp(join(tmpdir(), "ghostr-e2e-safe-"));
  const configured = join(root, "rust", "target", "artifacts");
  await assert.rejects(
    createRunFiles(root, {VIDEO_USER_E2E_ARTIFACT_ROOT: configured}),
    /outside Cargo target/,
  );
  await rm(root, {recursive: true, force: true});
});
