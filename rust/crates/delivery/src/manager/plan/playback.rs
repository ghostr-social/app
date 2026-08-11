use super::DeliveryState;
use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_engine::media_timeline::PlaybackWindow;
use ghostr_engine::playback::{
    MediaConsumption, PlaybackObservation, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::PostId;
use std::collections::HashMap;
use std::time::Duration;

mod targets;
use targets::{targets, TargetInputs};

pub(crate) struct PlaybackPlanInputs<'a> {
    pub(crate) stats: &'a HostStats,
    pub(crate) urls: &'a HashMap<PostId, String>,
    pub(crate) observed_at_ms: u64,
    pub(crate) demanded_end: Option<u64>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlaybackPlan {
    frontier: Option<(PostId, u64)>,
    media: Option<(PostId, PlaybackWindow)>,
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

    pub(crate) fn media_window(&self, post: &PostId) -> Option<PlaybackWindow> {
        self.media
            .as_ref()
            .filter(|(active, _)| active == post)
            .map(|(_, window)| *window)
    }
}

pub(crate) fn playback_plan(
    state: &mut DeliveryState,
    inputs: PlaybackPlanInputs<'_>,
) -> PlaybackPlan {
    let Some(snapshot) = snapshot(state) else {
        return PlaybackPlan::default();
    };
    let timed = state
        .catalog()
        .lookup(&snapshot.post)
        .is_some_and(|entry| entry.timeline().is_some());
    if !refills(snapshot.observation.phase()) {
        return existing_plan(snapshot, timed);
    }
    let bitrate = state
        .catalog()
        .estimated_bitrate(&snapshot.post, state.params());
    let media = MediaConsumption::new(bitrate, snapshot.observation.playback_rate_milli());
    let host = inputs.urls.get(&snapshot.post).and_then(|url| host_of(url));
    let targets = targets(
        state,
        TargetInputs {
            stats: inputs.stats,
            host: host.as_deref(),
            media,
            observed_at_ms: inputs.observed_at_ms,
        },
    );
    let byte_request = target_end(
        state,
        ByteTarget {
            snapshot: &snapshot,
            bitrate,
            steady: targets.steady,
            demanded_end: inputs.demanded_end,
        },
    );
    let byte_end = state
        .playback_mut()
        .authorize_bytes(&snapshot.session, byte_request)
        .unwrap_or(byte_request);
    let media_request = target_media_ms(&snapshot, targets.steady);
    let media_end = state
        .playback_mut()
        .authorize_media_ms(&snapshot.session, media_request)
        .unwrap_or(media_request);
    let emergency = snapshot.observation.needs_urgent_refill(
        targets.inflow_bits_per_second,
        media,
        targets.emergency,
    );
    build_plan(PlanShape {
        post: snapshot.post,
        byte_end,
        media_end_ms: media_end,
        timed,
        emergency,
    })
}

#[derive(Clone)]
struct PlaybackSnapshot {
    session: PlaybackSession,
    post: PostId,
    observation: PlaybackObservation,
    authorized_end: u64,
    authorized_media_ms: u64,
}

fn snapshot(state: &DeliveryState) -> Option<PlaybackSnapshot> {
    let status = state.playback();
    let session = status.session()?.clone();
    Some(PlaybackSnapshot {
        post: session.post().clone(),
        session,
        observation: status.observation()?,
        authorized_end: status.authorized_end()?,
        authorized_media_ms: status.authorized_media_ms()?,
    })
}

fn existing_plan(snapshot: PlaybackSnapshot, timed: bool) -> PlaybackPlan {
    build_plan(PlanShape {
        post: snapshot.post,
        byte_end: snapshot.authorized_end,
        media_end_ms: snapshot.authorized_media_ms,
        timed,
        emergency: false,
    })
}

struct PlanShape {
    post: PostId,
    byte_end: u64,
    media_end_ms: u64,
    timed: bool,
    emergency: bool,
}

fn build_plan(shape: PlanShape) -> PlaybackPlan {
    let frontier = (!shape.timed).then(|| (shape.post.clone(), shape.byte_end));
    let media = shape
        .timed
        .then(|| PlaybackWindow::try_new(0, shape.media_end_ms).ok())
        .flatten()
        .map(|window| (shape.post, window));
    PlaybackPlan {
        frontier,
        media,
        emergency: shape.emergency,
    }
}

fn refills(phase: PlaybackPhase) -> bool {
    matches!(
        phase,
        PlaybackPhase::Starting | PlaybackPhase::Playing | PlaybackPhase::NetworkStalled
    )
}

struct ByteTarget<'a> {
    snapshot: &'a PlaybackSnapshot,
    bitrate: u64,
    steady: Duration,
    demanded_end: Option<u64>,
}

fn target_end(state: &DeliveryState, target: ByteTarget<'_>) -> u64 {
    let horizon = target
        .snapshot
        .observation
        .position()
        .saturating_add(target.steady);
    let bits = horizon
        .as_millis()
        .saturating_mul(u128::from(target.bitrate));
    let media_end = (bits / 8_000).min(u128::from(u64::MAX)) as u64;
    let requested = media_end.max(target.demanded_end.unwrap_or(0));
    state
        .catalog()
        .lookup(&target.snapshot.post)
        .and_then(|entry| entry.total_bytes())
        .map_or(requested, |total| requested.min(total))
}

fn target_media_ms(snapshot: &PlaybackSnapshot, steady: Duration) -> u64 {
    snapshot
        .observation
        .position()
        .saturating_add(steady)
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}
