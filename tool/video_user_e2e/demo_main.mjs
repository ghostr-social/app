import {pathToFileURL} from "node:url";
import {runVideoDemo} from "./demo_session.mjs";

export async function runVideoDemoMain(input = {}) {
  await runVideoDemo({
    root: input.root ?? process.cwd(),
    environment: input.environment ?? process.env,
  });
}

export function reportVideoDemoFailure(error, output = console.error) {
  output(`video-demo: ${error.message}`);
  process.exitCode = 1;
}

const direct = Boolean(process.argv[1])
  && pathToFileURL(process.argv[1]).href === import.meta.url;
if (direct) runVideoDemoMain().catch(reportVideoDemoFailure);
