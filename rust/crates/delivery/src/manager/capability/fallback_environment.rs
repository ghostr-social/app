use crate::cache_registry::CacheRegistry;
use crate::debug::network::NetworkThrottle;
use crate::delivery_events::{DeliveryCandidate, DeliveryFocus, FocusItem};
use crate::manager::{DeliveryManagerConfig, DeliveryTuning};
use crate::segmented::SegmentedCache;
use core::sync::atomic::{AtomicU64, Ordering};
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

pub(super) struct TestRoot(pub(super) PathBuf);

impl Drop for TestRoot {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).ok();
    }
}

pub(super) fn test_root() -> TestRoot {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let suffix = NEXT.fetch_add(1, Ordering::Relaxed);
    TestRoot(
        std::env::temp_dir().join(format!("ghostr-capability-{}-{suffix}", std::process::id())),
    )
}

pub(super) fn store(root: &Path) -> Arc<PartialRangeStore> {
    Arc::new(PartialRangeStore::with_capacity(
        root.to_owned(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ))
}

pub(super) fn config(store: Arc<PartialRangeStore>, stats_path: PathBuf) -> DeliveryManagerConfig {
    let limits = MediaRequestLimits::try_new(1, 1).expect("request limits");
    DeliveryManagerConfig {
        store,
        requests: MediaRequestExecutor::new(Arc::new(NoRequest), limits),
        cache: CacheRegistry::new(),
        segmented: SegmentedCache::new(),
        network: NetworkThrottle::new(),
        network_status: crate::delivery_events::DeliveryNetworkStatus::unavailable(),
        stats_path,
        params: EngineParams::default(),
        level: DataUsageLevel::Balanced,
        tuning: DeliveryTuning::default(),
        transform: None,
    }
}

pub(super) fn focus_candidate(post: PostId) -> (DeliveryFocus, DeliveryCandidate) {
    let high = rendition("malformed");
    let focus = DeliveryFocus::compatibility(
        vec![FocusItem {
            post: post.clone(),
            meta: high.meta().clone(),
        }],
        0,
        0,
    );
    let candidate = DeliveryCandidate {
        post,
        meta: high.meta().clone(),
        preview: None,
        metadata_evidence: Vec::new(),
        renditions: vec![high, rendition("fallback")],
        discovered_at: 1,
    };
    (focus, candidate)
}

fn rendition(name: &str) -> VideoRendition {
    let meta = VideoMeta {
        urls: vec![format!("https://{name}.example/video.mp4")],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(2_000),
    };
    VideoRendition::try_new(meta, None).expect("rendition")
}

struct NoRequest;

impl MediaHttpRequests for NoRequest {
    fn get(&self, _url: &str) -> anyhow::Result<reqwest::RequestBuilder> {
        anyhow::bail!("network forbidden")
    }
}
