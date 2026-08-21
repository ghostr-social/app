use super::support::{chunk_request, transfer_identity};
use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::{ChunkAttempt, CompletionStatus, InFlightChunks};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[test]
fn completing_required_hedge_bytes_cancels_the_losing_request() {
    let mut active = InFlightChunks::new();
    let post = PostId::new("video");
    let chunk = ChunkId {
        post: post.clone(),
        range: ByteRange::new(0, 64_000),
    };
    let (primary, loser) = insert(&mut active, &chunk, "https://a.example/v", None);
    let (hedge, _hedge_token) = insert(
        &mut active,
        &chunk,
        "https://b.example/v",
        Some(primary.id()),
    );

    assert!(active.complete_hedge_winner(hedge.id()));
    assert_eq!(active.finish(&hedge), CompletionStatus::HedgeWinner);
    assert!(loser.is_cancelled());
    assert_eq!(active.finish(&primary), CompletionStatus::HedgeLoser);
}

fn insert(
    active: &mut InFlightChunks,
    chunk: &ChunkId,
    source: &str,
    primary: Option<ghostr_engine::ActionId>,
) -> (ChunkAttempt, crate::chunk::cancel::CancelToken) {
    let attempt = active.next_attempt(chunk.clone(), transfer_identity(&chunk.post, source));
    let (handle, token) = cancel_pair();
    active.insert(
        &attempt,
        chunk_request(chunk.clone(), PreemptionAuthority::Transition),
        source.into(),
        0,
        handle,
    );
    if let Some(primary) = primary {
        assert!(active.link_hedge(primary, attempt.id()));
    }
    (attempt, token)
}
