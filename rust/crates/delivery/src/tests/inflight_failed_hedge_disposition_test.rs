use super::support::{chunk_request, transfer_identity};
use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::{CompletionStatus, InFlightChunks};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[test]
fn a_hedge_that_finishes_without_winning_is_an_explicit_loser() {
    let mut active = InFlightChunks::new();
    let chunk = ChunkId {
        post: PostId::new("video"),
        range: ByteRange::new(0, 64_000),
    };
    let primary = insert(&mut active, &chunk, "https://a.example/v");
    let alternate = insert(&mut active, &chunk, "https://b.example/v");
    assert!(active.link_hedge(primary.id(), alternate.id()));

    assert_eq!(active.finish(&alternate), CompletionStatus::HedgeLoser);
}

fn insert(
    active: &mut InFlightChunks,
    chunk: &ChunkId,
    source: &str,
) -> crate::manager::inflight::ChunkAttempt {
    let attempt = active.next_attempt(chunk.clone(), transfer_identity(&chunk.post, source));
    let (handle, _) = cancel_pair();
    active.insert(
        &attempt,
        chunk_request(chunk.clone(), PreemptionAuthority::Transition),
        source.into(),
        0,
        handle,
    );
    attempt
}
