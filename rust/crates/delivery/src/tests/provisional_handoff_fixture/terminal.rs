use super::provenance::acknowledged_full_roster_handoff_state_at;
use super::CURRENT;
use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::{ActiveAction, InFlightChunks};
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::scheduling::RangeRequest;
use ghostr_engine::{ByteRange, ChunkId, DataUsageLevel, PostId};

const CURRENT_SOURCE: &str = "https://current.example/current.mp4";

pub(in crate::tests) fn terminal_current_handoff_state(
) -> (DeliveryState, [ActiveAction; 3]) {
    let (state, mut active) =
        acknowledged_full_roster_handoff_state_at(DataUsageLevel::Balanced);
    active[2] = terminal_action(&state, CURRENT, CURRENT_SOURCE, 3);
    (state, active)
}

pub(in crate::tests) fn terminal_future_handoff_state(
) -> (DeliveryState, [ActiveAction; 3]) {
    let (state, mut active) =
        acknowledged_full_roster_handoff_state_at(DataUsageLevel::Balanced);
    active[0] = terminal_action(&state, "third", "https://media.example/third.mp4", 1);
    (state, active)
}

fn terminal_action(state: &DeliveryState, post: &str, source: &str, action: u64) -> ActiveAction {
    let chunk = ChunkId {
        post: PostId::new(post),
        range: ByteRange::new(0, 65_536),
    };
    let identity = state
        .catalog()
        .transfer_identity(&chunk.post, source)
        .expect("terminal representation");
    let mut inflight = InFlightChunks::new();
    let attempt = (0..action)
        .map(|_| inflight.next_attempt(chunk.clone(), identity.clone()))
        .last()
        .expect("positive action id");
    let (handle, _) = cancel_pair();
    inflight.insert(&attempt, request(chunk), host(source), 4_000, handle);
    attempt.mark_io_finished();
    inflight.actions().pop().expect("terminal action")
}

fn request(chunk: ChunkId) -> RangeRequest {
    let authority = if chunk.post == PostId::new(CURRENT) {
        PreemptionAuthority::PlaybackCritical
    } else {
        PreemptionAuthority::Transition
    };
    RangeRequest {
        chunk,
        authority,
        score: 1.0,
        contiguous_depth_bytes: 0,
    }
}

fn host(source: &str) -> String {
    source
        .strip_prefix("https://")
        .and_then(|remainder| remainder.split('/').next())
        .expect("fixture host")
        .to_owned()
}
