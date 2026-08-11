use super::support::transfer_identity;
use crate::chunk::cancel::{cancel_pair, CancelToken};
use crate::manager::inflight::{ChunkAttempt, CompletionStatus, InFlightChunks};
use crate::manager::plan::PlannedTransfer;
use ghostr_engine::scoring::ChunkRequest;
use ghostr_engine::tiers::Tier;
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[test]
fn finished_fence_does_not_evict_live_protected_work() {
    let current = transfer("current", 0, Tier::T0PlaybackEmergency, 0);
    let next = transfer("next", 64, Tier::T2Startability, 64);
    let farther = transfer("farther", 0, Tier::T2Startability, 0);
    let adjacent = transfer("current", 64, Tier::T0PlaybackEmergency, 64);
    let mut active = InFlightChunks::new();
    let (attempt, _) = insert(&mut active, &current);
    let (_, next_token) = insert(&mut active, &next);
    let (_, farther_token) = insert(&mut active, &farther);
    attempt.mark_io_finished();
    let planned = [adjacent, next, farther];

    active.reconcile(&planned, 3);
    let priority: Vec<_> = planned
        .iter()
        .map(|work| work.request.chunk.clone())
        .collect();
    active.preempt_for_current(&PostId::new("current"), &priority, 3);

    assert!(!next_token.is_cancelled());
    assert!(!farther_token.is_cancelled());
    assert_eq!(active.finish(&attempt), CompletionStatus::Current);
}

#[test]
fn finished_fence_is_excluded_from_live_accounting() {
    let current = transfer("current", 0, Tier::T0PlaybackEmergency, 0);
    let unrelated = transfer("other", 0, Tier::T2Startability, 0);
    let mut active = InFlightChunks::new();
    let (attempt, _) = insert(&mut active, &current);
    attempt.mark_io_finished();

    assert_eq!(active.len(), 0);
    assert_eq!(active.foreground_len(), 0);
    assert!(active.active_hosts().is_empty());
    assert!(!active.contains(&unrelated.request.chunk));
    assert_eq!(active.finish(&attempt), CompletionStatus::Current);
}

#[test]
fn finished_attempt_fences_adjacent_same_post_work_until_absorbed() {
    let first = transfer("current", 0, Tier::T0PlaybackEmergency, 0);
    let adjacent = transfer("current", 64, Tier::T0PlaybackEmergency, 64);
    let mut active = InFlightChunks::new();
    let (attempt, _) = insert(&mut active, &first);
    attempt.mark_io_finished();

    assert!(active.contains(&adjacent.request.chunk));
    assert_eq!(active.finish(&attempt), CompletionStatus::Current);
    assert!(!active.contains(&adjacent.request.chunk));
}

fn insert(active: &mut InFlightChunks, transfer: &PlannedTransfer) -> (ChunkAttempt, CancelToken) {
    let attempt = active.next_attempt(transfer.request.chunk.clone(), transfer.identity.clone());
    let (handle, token) = cancel_pair();
    active.insert(
        &attempt,
        transfer.request.clone(),
        "shared.example".to_owned(),
        handle,
    );
    (attempt, token)
}

fn transfer(post: &str, start: u64, tier: Tier, depth: u64) -> PlannedTransfer {
    let post = PostId::new(post);
    let url = format!("https://shared.example/{}.mp4", post.as_str());
    PlannedTransfer {
        identity: transfer_identity(&post, &url),
        request: ChunkRequest {
            chunk: ChunkId {
                post,
                range: ByteRange::new(start, start + 64),
            },
            tier,
            score: 1.0,
            startup_depth_bytes: depth,
        },
        url,
    }
}
