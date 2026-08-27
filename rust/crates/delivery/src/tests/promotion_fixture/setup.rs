use super::super::support::{chunk_request, temp_directory};
use crate::chunk::cancel::{cancel_pair, CancelToken};
use crate::manager::inflight::{ActionRegistration, ChunkAttempt, InFlightChunks};
use ghostr_engine::adaptive::{PreemptionAuthority, PromotionGrant, RetrievalRequest};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::{ByteRange, ChunkId, DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::{PartialRangeStore, StoreAction};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub(super) struct StoreSetup {
    pub(super) root: PathBuf,
    pub(super) store: PartialRangeStore,
    pub(super) identity: TransferIdentity,
    pub(super) action: StoreAction,
}

pub(super) async fn store_setup() -> StoreSetup {
    let root = temp_directory("promotion-lifecycle");
    let store = PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(16),
    );
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let identity = binding.transfer("https://origin.test/video").expect("valid test fixture");
    store.bind_representation(binding).await.expect("valid test fixture");
    let action = store.reserve_action(&identity, 1, 4).await.expect("valid test fixture");
    StoreSetup {
        root,
        store,
        identity,
        action,
    }
}

pub(super) fn registered(
    identity: &TransferIdentity,
    action: &StoreAction,
    grant: PromotionGrant,
) -> (InFlightChunks, ChunkAttempt, CancelToken) {
    let mut active = InFlightChunks::new();
    let chunk = ChunkId {
        post: PostId::new("post"),
        range: ByteRange::new(4, 8),
    };
    let attempt = active.next_attempt(chunk.clone(), identity.clone());
    let (handle, token) = cancel_pair();
    active.insert_action(ActionRegistration {
        attempt: &attempt,
        priority: chunk_request(chunk, PreemptionAuthority::Transition),
        retrieval: range(grant),
        host: "origin.test".into(),
        committed_until_ms: 0,
        launched_at_ms: 0,
        handle,
        store_action: Some(action.clone()),
        committed_network_bytes: Some(4),
        admission_claim: None,
    });
    (active, attempt, token)
}

fn range(promotion: PromotionGrant) -> RetrievalRequest {
    RetrievalRequest::FetchRange {
        bytes: ByteRange::new(4, 8),
        promotion: Some(promotion),
    }
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://origin.test/video".into()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(1_000),
    }
}
