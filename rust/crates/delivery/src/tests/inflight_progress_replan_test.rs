use super::support::{chunk_request, transfer_identity};
use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::InFlightChunks;
use crate::manager::plan::PlannedTransfer;
use ghostr_engine::tiers::Tier;
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[test]
fn stored_prefix_progress_keeps_the_overlapping_origin_request_in_flight() {
    let post = PostId::new("playing");
    let url = "https://media.example/playing.mp4";
    let identity = transfer_identity(&post, url);
    let active_chunk = chunk(post.clone(), 0, 256);
    let wanted_chunk = chunk(post.clone(), 64, 320);
    let mut active = InFlightChunks::new();
    let attempt = active.next_attempt(active_chunk.clone(), identity.clone());
    let (handle, token) = cancel_pair();
    active.insert(
        &attempt,
        chunk_request(active_chunk, Tier::T2Startability),
        "media.example".to_owned(),
        handle,
    );
    let transfer = PlannedTransfer {
        identity,
        request: chunk_request(wanted_chunk.clone(), Tier::T0PlaybackEmergency),
        url: url.to_owned(),
    };

    active.reconcile(&[transfer], 1);

    assert!(!token.is_cancelled());
    assert!(active.contains(&wanted_chunk));
}

#[test]
fn foreground_promotion_retains_a_narrower_startup_seed() {
    for tier in [Tier::T0PlaybackEmergency, Tier::T1CurrentTail] {
        assert_foreground_supersedes_seed(tier);
    }
}

fn assert_foreground_supersedes_seed(tier: Tier) {
    let post = PostId::new("playing");
    let url = "https://media.example/playing.mp4";
    let identity = transfer_identity(&post, url);
    let seed = chunk(post.clone(), 0, 64);
    let foreground = chunk(post, 0, 256);
    let mut active = InFlightChunks::new();
    let attempt = active.next_attempt(seed.clone(), identity.clone());
    let (handle, token) = cancel_pair();
    active.insert(
        &attempt,
        chunk_request(seed, Tier::T2Startability),
        "media.example".to_owned(),
        handle,
    );
    let planned = PlannedTransfer {
        identity,
        request: chunk_request(foreground.clone(), tier),
        url: url.to_owned(),
    };

    active.reconcile(&[planned], 2);

    assert!(!token.is_cancelled());
    assert!(active.contains(&foreground));
}

fn chunk(post: PostId, start: u64, end: u64) -> ChunkId {
    ChunkId {
        post,
        range: ByteRange::new(start, end),
    }
}
