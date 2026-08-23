use super::support::{chunk_request, transfer_identity};
use crate::chunk::cancel::cancel_pair;
use crate::chunk::downloader::ResponseObservation;
use crate::manager::inflight::{ActionRegistration, InFlightChunks};
use ghostr_engine::adaptive::{PreemptionAuthority, PromotionGrant, RetrievalRequest};
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[test]
fn coherent_206_releases_the_unused_promotion_reservation() {
    let post = PostId::new("post");
    let bytes = ByteRange::new(0, 4);
    let chunk = ChunkId {
        post: post.clone(),
        range: bytes,
    };
    let identity = transfer_identity(&post, "https://origin.test/video.mp4");
    let request = RetrievalRequest::FetchRange {
        bytes,
        promotion: Some(PromotionGrant {
            maximum_bytes: 16,
            valid_until_ms: u64::MAX,
        }),
    };
    let mut active = InFlightChunks::new();
    let attempt = active.next_attempt(chunk.clone(), identity);
    let (handle, _) = cancel_pair();
    active.insert_action(ActionRegistration {
        attempt: &attempt,
        priority: chunk_request(chunk, PreemptionAuthority::Transition),
        retrieval: request,
        host: "origin.test".into(),
        committed_until_ms: 10_000,
        launched_at_ms: 0,
        handle,
        store_action: None,
        committed_network_bytes: None,
    });

    assert!(active.observe_response(
        &attempt,
        ResponseObservation::Partial {
            range: bytes,
            total: Some(16),
        },
    ));
    let snapshot = active.actions().remove(0);
    assert_eq!(
        snapshot.request(),
        RetrievalRequest::FetchRange {
            bytes,
            promotion: None
        }
    );
    assert_eq!(snapshot.effective_bytes(), bytes);
    assert_eq!(snapshot.reserved_storage_bytes(), 4);
}
