use super::super::issue;
use crate::progressive::capabilities::ProgressiveCapabilities;
use crate::progressive::route::{ProgressiveState, ProgressiveTiming};
use ghostr_delivery::cache_registry::{CacheStatus, CacheVideo};
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_delivery::playback_demand::demand_channel;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

#[tokio::test]
async fn issuance_wakes_when_cache_publishes_exact_authority() {
    let (state, root) = state();
    let meta = meta();
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta.clone());
    state.store.bind_representation(binding).await.unwrap();
    state.store.set_total_len("clip", 16).await.unwrap();
    state.cache.insert("clip");
    let issued_state = state.clone();
    let issued_meta = meta.clone();
    let mut issued =
        tokio::spawn(async move { issue(&issued_state, "clip", &issued_meta).await.unwrap() });
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
        .unwrap()
        .unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

fn state() -> (Arc<ProgressiveState>, PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "ghostr-progressive-cache-wake-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
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
