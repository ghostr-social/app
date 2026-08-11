import {runVideoUserE2e} from "./runner.mjs";
import {verifyPinnedBrowser} from "./verified_browser.mjs";

const CHECK_ONLY = "--check-prerequisites";

async function main() {
  const arguments_ = process.argv.slice(2);
  if (arguments_.some((argument) => argument !== CHECK_ONLY)) {
    throw new Error(`unknown video-user-e2e argument: ${arguments_.join(", ")}`);
  }
  const browser = await verifyPinnedBrowser({environment: process.env});
  console.log(`Pinned browser verified: ${browser.version} (${browser.sha256})`);
  if (arguments_.includes(CHECK_ONLY)) return;
  const result = await runVideoUserE2e({
    root: process.cwd(), environment: process.env, browser,
  });
  console.log(`Local video user E2E passed; artifacts: ${result.artifacts}`);
}

main().catch((error) => {
  console.error(`video-user-e2e: ${error.message}`);
  process.exitCode = 1;
});
