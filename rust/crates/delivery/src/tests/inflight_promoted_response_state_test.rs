use super::support::{chunk_request, transfer_identity};
use crate::chunk::cancel::cancel_pair;
use crate::chunk::downloader::ResponseObservation;
use crate::manager::inflight::{ActionRegistration, InFlightChunks};
use ghostr_engine::adaptive::{
    PreemptionAuthority, PromotionGrant, RetrievalRequest, WholeBodyContract, WholeFetchReason,
};
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[test]
fn promoted_200_expands_effective_inflight_coverage_at_headers() {
    let post = PostId::new("post");
    let bytes = ByteRange::new(4, 8);
    let chunk = ChunkId {
        post: post.clone(),
        range: bytes,
    };
    let identity = transfer_identity(&post, "https://origin.test/video.mp4");
    let launched = RetrievalRequest::FetchRange {
        bytes,
        promotion: Some(PromotionGrant {
            maximum_bytes: 16,
            valid_until_ms: u64::MAX,
        }),
    };
    let whole = RetrievalRequest::FetchWhole {
        contract: WholeBodyContract::Exact { expected_bytes: 16 },
        reason: WholeFetchReason::PromotedResponse,
    };
    let mut active = InFlightChunks::new();
    let attempt = active.next_attempt(chunk.clone(), identity);
    let (handle, _) = cancel_pair();
    active.insert_action(ActionRegistration {
        attempt: &attempt,
        priority: chunk_request(chunk, PreemptionAuthority::Transition),
        retrieval: launched,
        host: "origin.test".into(),
        committed_until_ms: 10_000,
        launched_at_ms: 0,
        handle,
        store_action: None,
        committed_network_bytes: None,
        admission_claim: None,
    });

    assert!(active.observe_response(
        &attempt,
        ResponseObservation::Body {
            request: whole,
            total: Some(16),
            range_support: Some(false),
            promoted: true,
        },
    ));
    let snapshot = active.actions().remove(0);
    assert_eq!(snapshot.request(), whole);
    assert_eq!(snapshot.effective_bytes(), ByteRange::new(0, 16));
    assert_eq!(snapshot.reserved_storage_bytes(), 16);
}
