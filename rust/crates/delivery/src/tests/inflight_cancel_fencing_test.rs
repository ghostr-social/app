use super::support::{chunk_request, transfer_identity};
use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::{CompletionStatus, InFlightChunks};
use ghostr_engine::tiers::Tier;
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[test]
fn a_cancelled_attempt_cannot_return_as_an_untracked_completion() {
    let mut active = InFlightChunks::new();
    let chunk = ChunkId {
        post: PostId::new("old"),
        range: ByteRange::new(0, 8),
    };
    let attempt = active.next_attempt(
        chunk.clone(),
        transfer_identity(&chunk.post, "https://slow.example/video"),
    );
    let (handle, _token) = cancel_pair();
    active.insert(
        &attempt,
        chunk_request(chunk.clone(), Tier::T4Speculative),
        "slow.example".to_owned(),
        handle,
    );

    assert!(active.cancel(&chunk));

    assert_eq!(active.finish(&attempt), CompletionStatus::Superseded);
}
