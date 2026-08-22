use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::{ActionRegistration, CompletionStatus, InFlightChunks};
use crate::manager::plan::PlannedTransfer;
use crate::tests::support::{chunk_request, transfer_identity};
use ghostr_engine::adaptive::{
    PreemptionAuthority, RetrievalRequest, WholeBodyContract, WholeFetchReason,
};
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[test]
fn planned_whole_cancels_an_overlapping_range_until_terminal_ack() {
    let post = PostId::new("post");
    let url = "https://origin.test/video.mp4";
    let identity = transfer_identity(&post, url);
    let chunk = ChunkId {
        post,
        range: ByteRange::new(0, 16),
    };
    let priority = chunk_request(chunk.clone(), PreemptionAuthority::Transition);
    let range = RetrievalRequest::FetchRange {
        bytes: chunk.range,
        promotion: None,
    };
    let whole = RetrievalRequest::FetchWhole {
        contract: WholeBodyContract::Exact { expected_bytes: 16 },
        reason: WholeFetchReason::PlannedCompletion,
    };
    let planned = PlannedTransfer {
        request: priority.clone(),
        retrieval: whole,
        url: url.to_owned(),
        identity: identity.clone(),
        commitment_until_ms: 0,
    };
    let mut active = InFlightChunks::new();
    let attempt = active.next_attempt(chunk, identity);
    let (handle, token) = cancel_pair();
    active.insert_action(ActionRegistration {
        attempt: &attempt,
        priority,
        retrieval: range,
        host: "origin.test".into(),
        committed_until_ms: 0,
        launched_at_ms: 0,
        handle,
        store_action: None,
    });

    active.reconcile(std::slice::from_ref(&planned), 1);

    assert!(token.is_cancelled());
    assert_eq!(active.len(), 1, "cancelling work retains its reservation");
    assert_ne!(planned.id().retrieval, range);
    assert_eq!(active.finish(&attempt), CompletionStatus::Cancelled);
}
