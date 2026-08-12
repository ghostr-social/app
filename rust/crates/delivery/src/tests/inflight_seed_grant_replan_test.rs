use super::support::transfer_identity;
use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::InFlightChunks;
use crate::manager::plan::PlannedTransfer;
use ghostr_engine::scoring::ChunkRequest;
use ghostr_engine::tiers::Tier;
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[test]
fn same_tier_replan_cannot_preempt_the_bounded_active_seed() {
    let seed = transfer("near", 0, 0, 1.0);
    let shifted = transfer("near", 16, 16, 1.0);
    let peer = transfer("far", 0, 0, 2.0);
    let mut active = InFlightChunks::new();
    let attempt = active.next_attempt(seed.request.chunk.clone(), seed.identity.clone());
    let (handle, token) = cancel_pair();
    active.insert(&attempt, seed.request, "media.example".to_owned(), handle);

    active.reconcile(&[shifted, peer], 1);

    assert!(!token.is_cancelled());
}

fn transfer(post: &str, start: u64, depth: u64, score: f64) -> PlannedTransfer {
    let post = PostId::new(post);
    let url = format!("https://media.example/{}.mp4", post.as_str());
    PlannedTransfer {
        identity: transfer_identity(&post, &url),
        request: ChunkRequest {
            chunk: ChunkId {
                post,
                range: ByteRange::new(start, start + 64),
            },
            tier: Tier::T2Startability,
            score,
            startup_depth_bytes: depth,
        },
        url,
    }
}
