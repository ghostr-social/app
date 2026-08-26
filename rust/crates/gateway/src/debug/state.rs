//! Serializable view of progressive delivery for the debug web app.

use crate::progressive::route::ProgressiveState;
use ghostr_delivery::debug::feed::{DebugFeedItem, DebugFeedSnapshot};
use ghostr_delivery::debug::network::NetworkProfile;
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_engine::host_stats::host_of;
use serde::Serialize;

mod plan;
mod video;
use plan::{snapshots as plan_snapshots, AdaptivePlanSnapshot};
use video::{snapshot as video_snapshot, VideoSnapshot};

#[derive(Debug, Serialize)]
pub struct DebugSnapshot {
    nostr: DebugFeedSnapshot,
    network: NetworkProfile,
    connections: Vec<ConnectionSnapshot>,
    storage: StorageSnapshot,
    adaptive_plans: Vec<AdaptivePlanSnapshot>,
    videos: Vec<VideoSnapshot>,
    hls_videos: Vec<HlsVideoSnapshot>,
}

#[derive(Debug, Serialize)]
pub struct ConnectionSnapshot {
    host: String,
    active: usize,
}

#[derive(Debug, Serialize)]
pub struct StorageSnapshot {
    used_bytes: u64,
    known_bytes: u64,
    video_count: usize,
    complete_count: usize,
}

#[derive(Debug, Serialize)]
pub struct HlsVideoSnapshot {
    id: String,
    nostr_event_id: String,
    title: Option<String>,
    creator: String,
    created_at: u64,
    source_host: Option<String>,
    source_count: usize,
    duration_ms: Option<u64>,
    delivery: &'static str,
    status: &'static str,
}

pub(super) async fn snapshot(state: &ProgressiveState, delivery: &DeliveryHandle) -> DebugSnapshot {
    let mut videos = Vec::new();
    for video in state.cache.videos() {
        videos.push(video_snapshot(state, video).await);
    }
    let used_bytes = state.store.used_bytes().await;
    DebugSnapshot {
        nostr: state.debug_feed.snapshot(),
        network: state.network.profile(),
        connections: connections(state),
        storage: storage(&videos, used_bytes),
        adaptive_plans: plan_snapshots(&delivery.plan_history()),
        hls_videos: state
            .debug_feed
            .hls_items()
            .into_iter()
            .map(hls_video_snapshot)
            .collect(),
        videos,
    }
}

fn hls_video_snapshot(item: DebugFeedItem) -> HlsVideoSnapshot {
    HlsVideoSnapshot {
        source_host: item.meta.urls.first().and_then(|url| host_of(url)),
        source_count: item.meta.urls.len(),
        duration_ms: item.meta.duration_ms,
        delivery: "hls",
        status: "stream",
        nostr_event_id: item.event_id,
        created_at: item.created_at,
        creator: item.creator,
        title: item.title,
        id: item.id,
    }
}

fn connections(state: &ProgressiveState) -> Vec<ConnectionSnapshot> {
    state
        .network
        .active_connections()
        .into_iter()
        .map(|(host, active)| ConnectionSnapshot { host, active })
        .collect()
}

fn storage(videos: &[VideoSnapshot], used_bytes: u64) -> StorageSnapshot {
    StorageSnapshot {
        used_bytes,
        known_bytes: videos.iter().filter_map(VideoSnapshot::total_bytes).sum(),
        video_count: videos.len(),
        complete_count: videos.iter().filter(|video| video.is_complete()).count(),
    }
}
