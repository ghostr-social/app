use super::support::{chunk_request, transfer_identity};
use crate::chunk::cancel::{cancel_pair, CancelToken};
use crate::manager::inflight::InFlightChunks;
use crate::manager::plan::PlannedTransfer;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::{ByteRange, ChunkId, PostId};
use std::collections::HashSet;

#[test]
fn only_the_exact_policy_retained_range_survives_plan_omission() {
    let first = transfer(0);
    let adjacent = transfer(96);
    let mut inflight = InFlightChunks::new();
    let first_token = insert(&mut inflight, &first);
    let adjacent_token = insert(&mut inflight, &adjacent);

    inflight.reconcile_with_commitments(&[], 2, &HashSet::from([first.id()]));

    assert!(!first_token.is_cancelled());
    assert!(adjacent_token.is_cancelled());
    inflight.reconcile_with_commitments(&[], 2, &HashSet::new());
    assert!(first_token.is_cancelled());
}

fn insert(inflight: &mut InFlightChunks, transfer: &PlannedTransfer) -> CancelToken {
    let attempt = inflight.next_attempt(transfer.request.chunk.clone(), transfer.identity.clone());
    let (handle, token) = cancel_pair();
    inflight.insert(
        &attempt,
        transfer.request.clone(),
        "a.example".into(),
        transfer.commitment_until_ms,
        handle,
    );
    token
}

fn transfer(start: u64) -> PlannedTransfer {
    let post = PostId::new("ahead");
    let url = "https://a.example/video".to_owned();
    PlannedTransfer {
        identity: transfer_identity(&post, &url),
        request: chunk_request(
            ChunkId {
                post,
                range: ByteRange::new(start, start + 96),
            },
            PreemptionAuthority::Transition,
        ),
        url,
        commitment_until_ms: 5_000,
    }
}
