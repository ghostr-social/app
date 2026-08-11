use super::support::{chunk_request, transfer_identity};
use crate::chunk::cancel::{cancel_pair, CancelToken};
use crate::manager::inflight::InFlightChunks;
use crate::manager::plan::eviction::ProtectedSeedEviction;
use crate::manager::plan::PlannedTransfer;
use ghostr_engine::tiers::Tier;
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[test]
fn typed_eviction_authority_controls_both_preemption_paths() {
    let [urgent, ahead] = transfers();
    let (mut deferred, deferred_token) = active(&ahead);
    deferred.reconcile_with_eviction(
        &[urgent.clone(), ahead.clone()],
        1,
        ProtectedSeedEviction::Defer,
    );
    deferred.preempt_for_current_with_eviction(
        &PostId::new("current"),
        &[urgent.request.chunk.clone(), ahead.request.chunk.clone()],
        1,
        ProtectedSeedEviction::Defer,
    );
    assert!(!deferred_token.is_cancelled());

    let (mut reservable, reserve_token) = active(&ahead);
    reservable.reconcile_with_eviction(
        &[urgent.clone(), ahead.clone()],
        1,
        ProtectedSeedEviction::Allow,
    );
    assert!(reserve_token.is_cancelled());

    let (mut preemptible, preempt_token) = active(&ahead);
    preemptible.preempt_for_current_with_eviction(
        &PostId::new("current"),
        &[urgent.request.chunk.clone(), ahead.request.chunk.clone()],
        1,
        ProtectedSeedEviction::Allow,
    );
    assert!(preempt_token.is_cancelled());

    let (mut unplanned, unplanned_token) = active(&ahead);
    unplanned.reconcile_with_eviction(&[urgent], 1, ProtectedSeedEviction::Defer);
    assert!(
        unplanned_token.is_cancelled(),
        "locality cancellation remains"
    );
}

fn transfers() -> [PlannedTransfer; 2] {
    [
        transfer("current", Tier::T0PlaybackEmergency),
        transfer("ahead", Tier::T2Startability),
    ]
}

fn transfer(post: &str, tier: Tier) -> PlannedTransfer {
    let post = PostId::new(post);
    let url = format!("https://{}.example/video", post.as_str());
    PlannedTransfer {
        identity: transfer_identity(&post, &url),
        request: chunk_request(
            ChunkId {
                post,
                range: ByteRange::new(0, 8),
            },
            tier,
        ),
        url,
    }
}

fn active(transfer: &PlannedTransfer) -> (InFlightChunks, CancelToken) {
    let mut active = InFlightChunks::new();
    let attempt = active.next_attempt(transfer.request.chunk.clone(), transfer.identity.clone());
    let (handle, token) = cancel_pair();
    active.insert(
        &attempt,
        transfer.request.clone(),
        "ahead.example".into(),
        handle,
    );
    (active, token)
}
