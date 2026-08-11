use super::DeliveryState;
use ghostr_engine::host_stats::{host_of, HostStats, OPTIMISTIC_THROUGHPUT_BPS};
use ghostr_engine::playback::{
    AdaptiveBufferPolicy, MediaConsumption, NetworkConditions, PlaybackObservation, PlaybackPhase,
    PlaybackSession,
};
use ghostr_engine::PostId;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlaybackPlan {
    frontier: Option<(PostId, u64)>,
    emergency: bool,
}

impl PlaybackPlan {
    pub(crate) fn tail_end(&self, post: &PostId) -> Option<u64> {
        self.frontier
            .as_ref()
            .filter(|(active, _)| active == post)
            .map(|(_, end)| *end)
    }

    pub(crate) fn emergency(&self) -> bool {
        self.emergency
    }
}

pub(crate) fn playback_plan(
    state: &mut DeliveryState,
    stats: &HostStats,
    urls: &HashMap<PostId, String>,
    observed_at_ms: u64,
    demanded_end: Option<u64>,
) -> PlaybackPlan {
    let Some(snapshot) = snapshot(state) else {
        return PlaybackPlan::default();
    };
    if !refills(snapshot.observation.phase()) {
        return existing_frontier(snapshot);
    }
    let bitrate = state
        .catalog()
        .estimated_bitrate(&snapshot.post, state.params());
    let media = MediaConsumption::new(bitrate, snapshot.observation.playback_rate_milli());
    let host = urls.get(&snapshot.post).and_then(|url| host_of(url));
    let targets = targets(state, stats, host.as_deref(), media, observed_at_ms);
    let requested = target_end(state, &snapshot, bitrate, targets.steady, demanded_end);
    let end = state
        .playback_mut()
        .authorize_bytes(&snapshot.session, requested)
        .unwrap_or(requested);
    PlaybackPlan {
        frontier: Some((snapshot.post, end)),
        emergency: snapshot.observation.needs_urgent_refill(
            targets.inflow_bits_per_second,
            media,
            targets.emergency,
        ),
    }
}

#[derive(Clone)]
struct PlaybackSnapshot {
    session: PlaybackSession,
    post: PostId,
    observation: PlaybackObservation,
    authorized_end: u64,
}

fn snapshot(state: &DeliveryState) -> Option<PlaybackSnapshot> {
    let status = state.playback();
    let session = status.session()?.clone();
    Some(PlaybackSnapshot {
        post: session.post().clone(),
        session,
        observation: status.observation()?,
        authorized_end: status.authorized_end()?,
    })
}

fn existing_frontier(snapshot: PlaybackSnapshot) -> PlaybackPlan {
    PlaybackPlan {
        frontier: Some((snapshot.post, snapshot.authorized_end)),
        emergency: false,
    }
}

fn refills(phase: PlaybackPhase) -> bool {
    matches!(
        phase,
        PlaybackPhase::Starting | PlaybackPhase::Playing | PlaybackPhase::NetworkStalled
    )
}

#[derive(Clone, Copy)]
struct Targets {
    steady: Duration,
    emergency: Duration,
    inflow_bits_per_second: u64,
}

fn targets(
    state: &DeliveryState,
    stats: &HostStats,
    host: Option<&str>,
    media: MediaConsumption,
    observed_at_ms: u64,
) -> Targets {
    let estimate = host
        .and_then(|value| stats.host_throughput(value))
        .or_else(|| stats.overall_throughput());
    let inflow = estimate
        .map(|value| value.bytes_per_second())
        .unwrap_or(OPTIMISTIC_THROUGHPUT_BPS);
    let Some(estimate) = estimate else {
        return fallback_targets(state, inflow);
    };
    let ttfb = host
        .and_then(|value| stats.expected_ttfb(value))
        .or_else(|| stats.overall_ttfb())
        .unwrap_or(Duration::from_millis(250));
    let target = AdaptiveBufferPolicy::default().target(
        NetworkConditions::from_estimate(estimate, ttfb, observed_at_ms),
        media,
    );
    Targets {
        steady: target.steady(),
        emergency: target.emergency_horizon(),
        inflow_bits_per_second: finite_bits(inflow),
    }
}

fn fallback_targets(state: &DeliveryState, inflow: f64) -> Targets {
    let params = state.params();
    Targets {
        steady: Duration::from_secs(
            params
                .emergency_buffer_s
                .max(params.head_seconds.saturating_mul(2)),
        ),
        emergency: Duration::from_secs(params.emergency_buffer_s),
        inflow_bits_per_second: finite_bits(inflow),
    }
}

fn target_end(
    state: &DeliveryState,
    snapshot: &PlaybackSnapshot,
    bitrate: u64,
    steady: Duration,
    demanded_end: Option<u64>,
) -> u64 {
    let horizon = snapshot.observation.position().saturating_add(steady);
    let bits = horizon.as_millis().saturating_mul(u128::from(bitrate));
    let media_end = (bits / 8_000).min(u128::from(u64::MAX)) as u64;
    let requested = media_end.max(demanded_end.unwrap_or(0));
    state
        .catalog()
        .lookup(&snapshot.post)
        .and_then(|entry| entry.total_bytes())
        .map_or(requested, |total| requested.min(total))
}

fn finite_bits(bytes_per_second: f64) -> u64 {
    if !bytes_per_second.is_finite() || bytes_per_second <= 0.0 {
        return 0;
    }
    (bytes_per_second * 8.0).min(u64::MAX as f64).round() as u64
}
