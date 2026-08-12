use super::support::{chunk_request, transfer_identity};
use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::InFlightChunks;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[test]
fn live_range_snapshot_carries_exact_identity_and_commitment_until_io_finishes() {
    let post = PostId::new("ahead");
    let url = "https://a.example/video";
    let identity = transfer_identity(&post, url);
    let chunk = ChunkId {
        post,
        range: ByteRange::new(10, 42),
    };
    let mut active = InFlightChunks::new();
    let attempt = active.next_attempt(chunk.clone(), identity.clone());
    let (handle, _) = cancel_pair();
    active.insert(
        &attempt,
        chunk_request(chunk.clone(), PreemptionAuthority::Speculative),
        "a.example".into(),
        5_000,
        handle,
    );

    let snapshot = active.ranges();
    assert_eq!(snapshot[0].chunk(), &chunk);
    assert_eq!(snapshot[0].identity(), &identity);
    assert_eq!(snapshot[0].committed_until_ms(), 5_000);
    attempt.mark_io_finished();
    assert!(active.ranges().is_empty());
}
