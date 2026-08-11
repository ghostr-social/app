export function validateJourney(trace, options = {}) {
  const limits = {
    maxJumpMs: options.maxJumpMs ?? 2_500,
    maxStalls: options.maxStalls ?? 2,
    minimumProgress: options.minimumProgress ?? 0.25,
  };
  requireLoopback(trace.requests);
  requireRangeEvidence(trace.requests);
  requirePlaybackProgress(trace.samples, trace.clicks, limits.minimumProgress);
  requireAheadBeforeEof(trace.samples);
  requireBoundedStalls(trace.samples, limits.maxStalls);
  requireFastJumps(trace.samples, trace.clicks, limits.maxJumpMs);
}

function requireLoopback(requests = []) {
  const allowed = new Set(["127.0.0.1", "[::1]"]);
  const leftLoopback = requests.some((entry) => {
    const url = new URL(entry.url);
    return url.protocol !== "data:" && !allowed.has(url.hostname);
  });
  if (leftLoopback) {
    throw new Error("journey left loopback");
  }
}

function requireRangeEvidence(requests = []) {
  const media = requests.filter((entry) => {
    return entry.method !== "HEAD" && new URL(entry.url).pathname === "/video.mp4";
  });
  if (!media.length) throw new Error("journey made no media requests");
  if (media.some((entry) => !entry.range || entry.status !== 206 || !entry.content_range)) {
    throw new Error("media request was not a truthful Range/206 response");
  }
  if (media.some((entry) => entry.failure && !entry.canceled)) {
    throw new Error("media response did not complete");
  }
  if (!media.some((entry) => entry.finished)) {
    throw new Error("media response did not complete");
  }
}

function requirePlaybackProgress(samples = [], clicks = [], minimum) {
  const ids = new Set([samples[0]?.player.id, ...clicks.map((click) => click.id)]);
  ids.delete(undefined);
  for (const id of ids) {
    const playing = samples.filter(
      (sample) => sample.player.id === id && sample.player.phase === "playing",
    );
    const first = playing[0]?.player.current_time;
    const last = playing.at(-1)?.player.current_time;
    if (first === undefined || last - first < minimum) {
      throw new Error(`${id} media time did not advance`);
    }
  }
}

function requireAheadBeforeEof(samples = []) {
  const observed = samples.some((sample) => {
    const current = sample.state.videos.find((video) => video.id === sample.player.id);
    const ahead = sample.state.videos.some(
      (video) => video.id !== sample.player.id && video.downloaded_bytes > 0,
    );
    return current && current.downloaded_bytes < current.total_bytes && ahead;
  });
  if (!observed) throw new Error("ahead work did not begin before current EOF");
}

function requireBoundedStalls(samples = [], maximum) {
  let stalls = 0;
  let previous = "";
  for (const sample of samples) {
    const phase = sample.player.phase;
    if (phase !== previous && ["buffering", "stalled"].includes(phase)) stalls += 1;
    previous = phase;
  }
  if (stalls > maximum) throw new Error(`playback stalled ${stalls} times`);
}

function requireFastJumps(samples = [], clicks = [], maximum) {
  for (const click of clicks) {
    const playing = samples.find(
      (sample) => sample.at_ms >= click.at_ms &&
        sample.player.id === click.id && sample.player.phase === "playing",
    );
    if (!playing || playing.at_ms - click.at_ms > maximum) {
      throw new Error(`${click.id} did not play within ${maximum} ms of the visible jump`);
    }
  }
}
