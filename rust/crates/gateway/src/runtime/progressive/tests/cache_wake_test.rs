use super::super::issue;
use crate::progressive::capabilities::ProgressiveCapabilities;
use crate::progressive::route::{ProgressiveState, ProgressiveTiming};
use core::time::Duration;
use ghostr_delivery::cache_registry::{CacheStatus, CacheVideo};
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_delivery::playback_demand::demand_channel;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

#[tokio::test]
async fn issuance_wakes_when_cache_publishes_exact_authority() {
    let (state, root) = state();
    let meta = meta();
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta.clone());
    state
        .store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    state
        .store
        .set_total_len("clip", 16)
        .await
        .expect("valid test fixture");
    state.cache.insert("clip");
    let issued_state = std::sync::Arc::clone(&state);
    let issued_meta = meta.clone();
    let mut issued = tokio::spawn(async move {
        issue(&issued_state, "clip", &issued_meta)
            .await
            .expect("valid test fixture")
    });
    assert!(tokio::time::timeout(Duration::from_millis(25), &mut issued)
        .await
        .is_err());

    state.cache.replace([CacheVideo {
        id: "clip".to_owned(),
        meta,
        status: CacheStatus::Ready,
    }]);
    tokio::time::timeout(Duration::from_secs(1), issued)
        .await
        .expect("valid test fixture")
        .expect("valid test fixture");
    std::fs::remove_dir_all(root).expect("valid test fixture");
}

fn state() -> (Arc<ProgressiveState>, PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "ghostr-progressive-cache-wake-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("valid test fixture")
            .as_nanos()
    ));
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ));
    let (demand, _) = demand_channel();
    let state = ProgressiveState {
        store,
        demand,
        cache: Default::default(),
        network: NetworkThrottle::new(),
        timing: ProgressiveTiming::default(),
        capabilities: ProgressiveCapabilities::production(),
    };
    (Arc::new(state), root)
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://media.example/clip.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(2_000),
    }
}
