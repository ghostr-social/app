use super::support::{chunk_request, transfer_identity};
use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::InFlightChunks;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[test]
fn live_action_snapshot_holds_its_reservation_until_terminal_ack() {
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

    let snapshot = active.actions();
    assert_eq!(snapshot[0].post(), &chunk.post);
    assert_eq!(snapshot[0].effective_bytes(), chunk.range);
    assert_eq!(snapshot[0].identity(), &identity);
    assert_eq!(snapshot[0].committed_until_ms(), 5_000);
    assert!(!snapshot[0].io_finished());
    attempt.mark_io_finished();
    let terminal = active.actions();
    assert_eq!(terminal.len(), 1);
    assert!(terminal[0].io_finished());
    active.finish(&attempt);
    assert!(active.actions().is_empty());
}
