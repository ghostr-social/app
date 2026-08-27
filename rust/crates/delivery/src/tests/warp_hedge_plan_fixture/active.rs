use super::{HedgeCase, PRIMARY};
use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::{ActionRegistration, ChunkAttempt, InFlightChunks};
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{PreemptionAuthority, RetrievalRequest};
use ghostr_engine::scheduling::RangeRequest;
use ghostr_engine::{ByteRange, ChunkId, PostId};

mod link;

#[derive(Clone, Copy)]
pub(super) struct Registration<'a> {
    attempt: &'a ChunkAttempt,
    source: &'a str,
    range: ByteRange,
    launched_at_ms: u64,
}

pub(super) fn actions(
    state: &DeliveryState,
    post: PostId,
    case: HedgeCase,
) -> Vec<crate::manager::inflight::ActiveAction> {
    let range = ByteRange::new(0, 64_000);
    let chunk = ChunkId {
        post,
        range,
    };
    let mut active = InFlightChunks::new();
    let primary = attempt(&mut active, state, chunk.clone(), PRIMARY);
    insert(
        &mut active,
        Registration {
            attempt: &primary,
            source: PRIMARY,
            range,
            launched_at_ms: case.primary_launched_at_ms(),
        },
    );
    link::alternate(
        &mut active,
        link::Input {
            state,
            chunk,
            primary: &primary,
            case,
        },
    );
    active.actions()
}

pub(super) fn attempt(
    active: &mut InFlightChunks,
    state: &DeliveryState,
    chunk: ChunkId,
    source: &str,
) -> ChunkAttempt {
    let identity = state
        .catalog()
        .transfer_identity(&chunk.post, source)
        .expect("valid test fixture");
    active.next_attempt(chunk, identity)
}

pub(super) fn insert(active: &mut InFlightChunks, registration: Registration<'_>) {
    let (handle, _) = cancel_pair();
    active.insert_action(ActionRegistration {
        attempt: registration.attempt,
        priority: RangeRequest {
            chunk: registration.attempt.chunk.clone(),
            authority: PreemptionAuthority::PlaybackCritical,
            score: 1.0,
            contiguous_depth_bytes: 0,
        },
        retrieval: RetrievalRequest::FetchRange {
            bytes: registration.range,
            promotion: None,
        },
        host: registration.source.into(),
        committed_until_ms: registration.launched_at_ms + 3_000,
        launched_at_ms: registration.launched_at_ms,
        handle,
        store_action: None,
        committed_network_bytes: None,
        admission_claim: None,
    });
}
