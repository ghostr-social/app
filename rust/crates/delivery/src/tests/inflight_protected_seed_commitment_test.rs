use super::support::{chunk_request, transfer_identity};
use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::InFlightChunks;
use crate::manager::plan::eviction::ProtectedSeedEviction;
use crate::manager::plan::PlannedTransfer;
use ghostr_engine::tiers::Tier;
use ghostr_engine::{ByteRange, ChunkId, PostId};
use std::collections::HashSet;

#[test]
fn only_identity_valid_started_seeds_survive_temporary_plan_omission() {
    let seed = transfer("ahead", "a", Tier::T2Startability);
    let urgent = transfer("ahead", "a", Tier::T0PlaybackEmergency);
    let protected = HashSet::from([seed.identity.clone()]);
    let (mut inflight, token) = active(&seed);

    inflight.reconcile_with_commitments(
        std::slice::from_ref(&urgent),
        1,
        ProtectedSeedEviction::Defer,
        &protected,
    );
    inflight.reconcile_with_commitments(&[], 1, ProtectedSeedEviction::Allow, &protected);
    assert!(!token.is_cancelled(), "promotion preserves seed commitment");

    inflight.reconcile_with_commitments(&[], 1, ProtectedSeedEviction::Allow, &HashSet::new());
    assert!(token.is_cancelled(), "leaving the protected set cancels IO");

    let (mut stale, stale_token) = active(&seed);
    let replacement = transfer_identity(&PostId::new("ahead"), "https://b.example/video");
    stale.reconcile_with_commitments(
        &[],
        1,
        ProtectedSeedEviction::Allow,
        &HashSet::from([replacement]),
    );
    assert!(stale_token.is_cancelled(), "identity changes cancel old IO");

    let (mut adjacent, adjacent_token) = active(&seed);
    let mut deepening = transfer("ahead", "a", Tier::T3Deepening);
    deepening.request.chunk.range = ByteRange::new(96, 128);
    adjacent.reconcile_with_commitments(&[deepening], 2, ProtectedSeedEviction::Allow, &protected);
    assert!(
        !adjacent_token.is_cancelled(),
        "adjacent nonforeground work preserves the paid seed"
    );

    let (mut adjacent, adjacent_token) = active(&seed);
    let mut foreground = urgent.clone();
    foreground.request.chunk.range = ByteRange::new(96, 128);
    adjacent.reconcile_with_commitments(&[foreground], 2, ProtectedSeedEviction::Allow, &protected);
    assert!(!adjacent_token.is_cancelled(), "adjacent T0 preserves IO");

    let (mut seeking, seek_token) = active(&seed);
    let mut gapped = urgent;
    gapped.request.chunk.range = ByteRange::new(97, 128);
    seeking.reconcile_with_commitments(&[gapped], 2, ProtectedSeedEviction::Allow, &protected);
    assert!(seek_token.is_cancelled(), "gapped demand cancels old IO");
}

fn active(transfer: &PlannedTransfer) -> (InFlightChunks, crate::chunk::cancel::CancelToken) {
    let mut active = InFlightChunks::new();
    let attempt = active.next_attempt(transfer.request.chunk.clone(), transfer.identity.clone());
    let (handle, token) = cancel_pair();
    active.insert(
        &attempt,
        transfer.request.clone(),
        "a.example".into(),
        handle,
    );
    (active, token)
}

fn transfer(post: &str, source: &str, tier: Tier) -> PlannedTransfer {
    let post = PostId::new(post);
    let url = format!("https://{source}.example/video");
    PlannedTransfer {
        identity: transfer_identity(&post, &url),
        request: chunk_request(
            ChunkId {
                post,
                range: ByteRange::new(0, 96),
            },
            tier,
        ),
        url,
    }
}
