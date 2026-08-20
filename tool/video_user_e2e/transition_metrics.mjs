export function protectedTransitionLatency(clicks = [], samples = []) {
  const protectedClicks = clicks.filter((click) => {
    return !click.superseded && click.protected_transition;
  });
  if (protectedClicks.length === 0) return 0;
  return Math.max(...protectedClicks.map((click) => presentedLatency(click, samples)));
}

function presentedLatency(click, samples) {
  const sample = samples.find((entry) => {
    return entry.at_ms >= click.at_ms
      && entry.player?.id === click.id
      && entry.player?.presented === true;
  });
  return sample ? sample.at_ms - click.at_ms : Number.POSITIVE_INFINITY;
}
