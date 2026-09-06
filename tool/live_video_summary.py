"""Callback timing summaries; missing frames have an unbounded upper latency."""
import math
import statistics


def callback_latency(sample):
    value = sample.get("firstFrameMs")
    if type(value) not in (int, float) or not math.isfinite(value) or value < 0:
        return math.inf
    return value


def distribution(values):
    if not values:
        return dict.fromkeys(("median", "p95", "worst"))
    ordered = sorted(values)
    measured = {"median": statistics.median(ordered),
                "p95": ordered[math.ceil(len(ordered) * 0.95) - 1],
                "worst": ordered[-1]}
    return {key: value if math.isfinite(value) else None
            for key, value in measured.items()}


def population(samples, requested):
    all_samples = samples + [{}] * max(0, requested - len(samples))
    latencies = [callback_latency(sample) for sample in all_samples]
    observed = [value for value in latencies if math.isfinite(value)]
    return {
        "requested": len(all_samples), "sampled": len(samples),
        "missing_first_frames": len(latencies) - len(observed),
        "over_100ms_or_missing": sum(value > 100 for value in latencies),
        "not_moving_or_missing": sum(s.get("renderedAndMoving") is not True
                                     for s in all_samples),
        "sampled_freezes_over_2s": sum((s.get("longestFreezeMs") or 0) > 2000
                                      for s in samples),
        "observed_callback_ms": distribution(observed),
        "all_requested_callback_ms": distribution(latencies),
    }


def pinned_population(report, pins):
    samples = [s for s in report["samples"] if s["scenario"] == "pinned_warp"]
    identifiers = [sample["eventId"] for sample in samples]
    if not pins or len(pins) != len(set(pins)):
        raise ValueError("Pinned summaries require distinct requested --pins")
    if len(identifiers) != len(set(identifiers)) or set(identifiers) - set(pins):
        raise ValueError("Pinned samples must uniquely match requested --pins")
    return population(samples, len(pins))


def last_record(report, kind):
    return next((r for r in reversed(report["records"]) if r.get("type") == kind), {})


def fresh_populations(report, expected):
    requested = last_record(report, "fresh_corpus").get("requested", expected)
    if type(requested) is not int or not 1 <= requested <= 1000:
        raise ValueError("Fresh requested count must be between 1 and 1000")
    scenarios = {name: [s for s in report["samples"] if s["scenario"] == name]
                 for name in ("startup", "browse", "warm_return", "rapid_final")}
    forward = scenarios["browse"]
    backward = scenarios["warm_return"]
    rapid = scenarios["rapid_final"]
    back_count = min(5, requested - 1)
    return {
        "cold_startup": population(scenarios["startup"], 1),
        "fresh": population(scenarios["startup"] + forward, requested),
        "scrolling": population(forward + backward + rapid, requested + back_count),
        "backward": population(backward, back_count),
        "rapid_settle": population(rapid, 1),
        "corpus": last_record(report, "fresh_corpus"),
    }


def measurement():
    return {
        "signal": "native_first_frame_callback", "physical_presentation_verified": False,
        "timing": "Focus activation to native first-frame callback, in ms.",
        "percentiles": "Median and nearest-rank p95; missing frames sort last.",
        "null_latency": "Unbounded because frames are missing, or no samples.",
        "limitations": "Callbacks and coarse position sampling do not establish "
                       "physical blank/frozen intervals or the visible 100 ms target. "
                       "Corpus exclusions and rapid intermediate gestures are not timed samples.",
    }


def summarize(report, mode, pins, expected=20):
    groups = ({"pinned": pinned_population(report, pins)} if mode == "pinned"
              else fresh_populations(report, expected))
    return {
        "schema_version": 1, "mode": mode, "target_ms": 100,
        "measurement": measurement(), **groups,
        "cache": {"cold": last_record(report, "configuration").get("coldCache"),
                  "key": last_record(report, "isolated_cache").get("key")},
        "failures": report.get("failures", []),
        "dropped_records": report.get("droppedRecords", 0),
        "gesture_releases": sum(r.get("type") == "gesture_release" for r in report["records"]),
        "excluded_candidates": [r for r in report["records"]
                                if r.get("type") == "corpus_excluded_video"],
        "direct_native_controls": [r for r in report["records"]
                                   if r.get("type") == "direct_player"],
    }
