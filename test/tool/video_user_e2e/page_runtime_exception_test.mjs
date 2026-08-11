import assert from "node:assert/strict";
import test from "node:test";
import {evaluate} from "../../../tool/video_user_e2e/page_runtime.mjs";
import {recordingPage} from "./page_runtime_support.mjs";

test("page evaluation surfaces the browser exception description", async () => {
  const {page} = recordingPage({
    exceptionDetails: {text: "evaluation failed", exception: {description: "visible failure"}},
  });

  await assert.rejects(evaluate(page, "throw new Error()"), /visible failure/);
});
