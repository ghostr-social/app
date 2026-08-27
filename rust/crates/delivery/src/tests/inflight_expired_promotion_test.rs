use super::support::{chunk_request, temp_directory};
use crate::chunk::cancel::cancel_pair;
use crate::chunk::downloader::{
    HttpResponseEvidence, OpenedResponse, ResponseObservation, ResponseWriteMode,
};
use crate::manager::inflight::{ActionRegistration, InFlightChunks};
use ghostr_engine::adaptive::{
    PreemptionAuthority, PromotionGrant, RetrievalRequest, WholeBodyContract, WholeFetchReason,
};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{ByteRange, ChunkId, DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn promotion_expired_at_headers_is_rejected_before_store_open() {
    let root = temp_directory("expired-promotion");
    let store = PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(16),
    );
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let identity = binding.transfer("https://origin.test/video").expect("valid test fixture");
    store.bind_representation(binding).await.expect("valid test fixture");
    let action = store.reserve_action(&identity, 1, 16).await.expect("valid test fixture");
    let bytes = ByteRange::new(4, 8);
    let chunk = ChunkId {
        post: PostId::new("post"),
        range: bytes,
    };
    let launched = RetrievalRequest::FetchRange {
        bytes,
        promotion: Some(PromotionGrant {
            maximum_bytes: 16,
            valid_until_ms: 100,
        }),
    };
    let mut active = InFlightChunks::new();
    let attempt = active.next_attempt(chunk.clone(), identity);
    let (handle, _) = cancel_pair();
    active.insert_action(ActionRegistration {
        attempt: &attempt,
        priority: chunk_request(chunk, PreemptionAuthority::Transition),
        retrieval: launched,
        host: "origin.test".into(),
        committed_until_ms: 0,
        launched_at_ms: 0,
        handle,
        store_action: Some(action.clone()),
        committed_network_bytes: None,
        admission_claim: None,
    });
    let whole = RetrievalRequest::FetchWhole {
        contract: WholeBodyContract::Exact { expected_bytes: 16 },
        reason: WholeFetchReason::PromotedResponse,
    };
    let response = OpenedResponse::new(
        ResponseObservation::Body {
            request: whole,
            total: Some(16),
            range_support: Some(false),
            promoted: true,
        },
        None,
        ResponseWriteMode::SingleResponse(WholeBodyContract::Exact { expected_bytes: 16 }),
        HttpResponseEvidence {
            final_url: "https://origin.test/video".into(),
            status: 200,
            content_type: Some("video/mp4".into()),
            validator: None,
            observed: 0.into(),
        },
    );

    assert!(!active.authorizes_response(&attempt, &action, &response, 101));
    assert!(store
        .media_snapshot("post")
        .await
        .expect("valid test fixture")
        .ranges()
        .is_empty());
    store.release_action(&action).await;
    std::fs::remove_dir_all(root).expect("valid test fixture");
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://origin.test/video".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(1_000),
    }
}
