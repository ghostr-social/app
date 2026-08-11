import {
  clickVideo, requireUserStartsPlayback, watchProgress, watchUntilPlaying,
} from "./browser_journey.mjs";
import {startBrowser} from "./browser_process.mjs";
import {sendControlAction} from "./impairment_executor.mjs";
import {startLocalOrigin} from "./local_origin.mjs";
import {refreshDebugSnapshot} from "./page_runtime.mjs";
import {registerOrderedVideos, selectVideoFocus} from "./ordered_admission.mjs";
import {createRunFiles, removeTransientRunFiles} from "./run_files.mjs";
import {startServer} from "./server_process.mjs";
import {writeFailure, writeSuccess} from "./trace_artifacts.mjs";
import {delay} from "./wait.mjs";

const DEFAULT_BOUNDARIES = Object.freeze({
  clickVideo,
  createRunFiles,
  delay,
  refreshDebugSnapshot,
  registerOrderedVideos,
  removeTransientRunFiles,
  requireUserStartsPlayback,
  selectVideoFocus,
  sendControlAction,
  startBrowser,
  startLocalOrigin,
  startServer,
  watchProgress,
  watchUntilPlaying,
  writeFailure,
  writeSuccess,
});

export function createRunnerBoundaries(overrides = {}) {
  return {...DEFAULT_BOUNDARIES, ...overrides};
}
