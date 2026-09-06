use crate::cache_registry::CacheRegistry;
use crate::debug::network::NetworkThrottle;
use crate::delivery_events::{command_channel, DeliveryFocus, DeliveryPlayback, FocusItem};
use crate::manager::resource_control::ResourceControl;
use crate::manager::{DeliveryManagerConfig, DeliveryTuning, DeliveryWorker};
use crate::playback_demand::demand_channel;
use crate::segmented::SegmentedCache;
use core::time::Duration;
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpClient;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

pub(super) async fn worker() -> (DeliveryWorker, PathBuf) {
    let root = std::env::temp_dir().join(format!("warp-stall-{}", rand::random::<u64>()));
    let config = configuration(&root);
    let (_, commands) = command_channel();
    let (_, demand) = demand_channel();
    let resources = ResourceControl::bootstrap(&config, tokio::time::Instant::now());
    let mut worker = DeliveryWorker::create(config, commands, demand, resources).await;
    worker.state.apply_focus(focus(), 0);
    (worker, root)
}

fn configuration(root: &Path) -> DeliveryManagerConfig {
    DeliveryManagerConfig {
        store: Arc::new(PartialRangeStore::with_capacity(
            root.to_owned(),
            Arc::new(Mutex::new(0)),
            StoreCapacity::system(u64::MAX),
        )),
        requests: MediaRequestExecutor::new(
            Arc::new(MediaHttpClient::public().expect("production HTTP client")),
            MediaRequestLimits::try_new(4, 4).expect("request limits"),
        ),
        cache: CacheRegistry::new(),
        segmented: SegmentedCache::new(),
        network: NetworkThrottle::new(),
        network_status: crate::delivery_events::DeliveryNetworkStatus::unavailable(),
        stats_path: root.join("stats.json"),
        params: EngineParams::default(),
        level: DataUsageLevel::Aggressive,
        tuning: DeliveryTuning::default(),
        transform: None,
    }
}

fn focus() -> DeliveryFocus {
    DeliveryFocus::compatibility(
        vec![FocusItem {
            post: PostId::new("current"),
            meta: VideoMeta {
                urls: vec!["https://media.example/video.mp4".into()],
                delivery: DeliveryKind::Progressive,
                sha256: None,
                size_bytes: Some(1_000_000),
                duration_ms: Some(10_000),
            },
        }],
        0,
        0,
    )
}

pub(super) fn update(generation: u64, sequence: u64, phase: PlaybackPhase) -> DeliveryPlayback {
    DeliveryPlayback {
        session: PlaybackSession::new(PostId::new("current"), generation),
        sequence: PlaybackObservationSequence::new(sequence),
        observation: PlaybackObservation::try_new(
            Duration::from_secs(1),
            Duration::from_millis(1_100),
            1_000,
            phase,
        )
        .expect("playback observation"),
    }
}
