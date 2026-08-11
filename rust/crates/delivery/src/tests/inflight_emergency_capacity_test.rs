use super::support::transfer_identity;
use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::InFlightChunks;
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[test]
fn an_emergency_frees_enough_speculative_slots_for_current_playback() {
    let current = PostId::new("current");
    let ahead = [
        chunk("ahead-a", 0),
        chunk("ahead-b", 8),
        chunk("ahead-c", 16),
    ];
    let mut active = InFlightChunks::new();
    let mut tokens = Vec::new();
    for chunk in &ahead {
        let attempt = active.next_attempt(
            chunk.clone(),
            transfer_identity(&chunk.post, "https://slow.example/video"),
        );
        let (handle, token) = cancel_pair();
        active.insert(&attempt, "slow.example".to_owned(), handle);
        tokens.push(token);
    }
    let priority = [
        chunk("current", 0),
        ahead[0].clone(),
        ahead[1].clone(),
        ahead[2].clone(),
    ];

    active.preempt_for_current(&current, &priority, 1);

    assert_eq!(
        active.len(),
        0,
        "the sole slot must be available to current playback"
    );
    assert!(tokens.iter().all(|token| token.is_cancelled()));
}

fn chunk(post: &str, start: u64) -> ChunkId {
    ChunkId {
        post: PostId::new(post),
        range: ByteRange::new(start, start + 8),
    }
}
