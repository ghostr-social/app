import {pathToFileURL} from "node:url";
import {parseVideoE2eArguments} from "./arguments.mjs";
import {runVideoUserE2e} from "./runner.mjs";
import {verifyPinnedBrowser} from "./verified_browser.mjs";

const DEFAULT_INPUT = {
  arguments: process.argv.slice(2),
  environment: process.env,
  root: process.cwd(),
  verify: verifyPinnedBrowser,
  run: runVideoUserE2e,
  log: console.log,
};

export async function runVideoE2eMain(input) {
  const options = parseVideoE2eArguments(input.arguments);
  const browser = await input.verify({environment: input.environment});
  input.log(`Pinned browser verified: ${browser.version} (${browser.sha256})`);
  if (options.checkOnly) return;
  const result = await input.run({
    root: input.root,
    environment: input.environment,
    browser,
    scenario: options.scenario,
  });
  input.log(`Local video user E2E passed; artifacts: ${result.artifacts}`);
}

export function reportVideoE2eFailure(error, output = console.error, processState = process) {
  output(`video-user-e2e: ${error.message}`);
  processState.exitCode = 1;
}

export function launchVideoE2eMain(input) {
  if (!input.direct) return;
  return runVideoE2eMain(input.main).catch(input.report ?? reportVideoE2eFailure);
}

launchVideoE2eMain({
  direct: Boolean(process.argv[1]) && pathToFileURL(process.argv[1]).href === import.meta.url,
  main: DEFAULT_INPUT,
});
