//! Serializable view of progressive delivery for the debug web app.

use crate::progressive::route::ProgressiveState;
use ghostr_delivery::cache_registry::CacheVideo;
use ghostr_delivery::debug::feed::{DebugFeedItem, DebugFeedSnapshot};
use ghostr_delivery::debug::network::NetworkProfile;
use ghostr_engine::host_stats::host_of;
use serde::Serialize;
use std::ops::Range;

#[derive(Debug, Serialize)]
pub struct DebugSnapshot {
    nostr: DebugFeedSnapshot,
    network: NetworkProfile,
    connections: Vec<ConnectionSnapshot>,
    storage: StorageSnapshot,
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
pub struct VideoSnapshot {
    id: String,
    nostr_event_id: Option<String>,
    title: Option<String>,
    creator: Option<String>,
    created_at: Option<u64>,
    source_host: Option<String>,
    source_count: usize,
    total_bytes: Option<u64>,
    downloaded_bytes: u64,
    duration_ms: Option<u64>,
    downloaded_duration_ms: Option<u64>,
    progress: Option<f64>,
    complete: bool,
    status: &'static str,
    ranges: Vec<RangeSnapshot>,
    playback_url: String,
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

#[derive(Debug, Serialize)]
pub struct RangeSnapshot {
    start: u64,
    end: u64,
}

pub(crate) async fn snapshot(state: &ProgressiveState) -> DebugSnapshot {
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

async fn video_snapshot(state: &ProgressiveState, video: CacheVideo) -> VideoSnapshot {
    let id = video.id;
    let metadata = state.debug_feed.metadata(&id);
    let total = state
        .store
        .total_len(&id)
        .await
        .unwrap_or(video.meta.size_bytes);
    let ranges = state.store.present_ranges(&id).await.unwrap_or_default();
    let downloaded = ranges.iter().map(range_len).sum();
    let complete = total.is_some_and(|size| downloaded >= size);
    VideoSnapshot {
        nostr_event_id: metadata.as_ref().map(|item| item.event_id.clone()),
        title: metadata.as_ref().and_then(|item| item.title.clone()),
        creator: metadata.as_ref().map(|item| item.creator.clone()),
        created_at: metadata.as_ref().map(|item| item.created_at),
        source_host: video.meta.urls.first().and_then(|url| host_of(url)),
        source_count: video.meta.urls.len(),
        downloaded_duration_ms: downloaded_duration(video.meta.duration_ms, downloaded, total),
        progress: progress(downloaded, total),
        status: status(downloaded, complete),
        playback_url: format!("/video.mp4?id={id}"),
        ranges: ranges.into_iter().map(range_snapshot).collect(),
        duration_ms: video.meta.duration_ms,
        downloaded_bytes: downloaded,
        total_bytes: total,
        complete,
        id,
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
        known_bytes: videos.iter().filter_map(|video| video.total_bytes).sum(),
        video_count: videos.len(),
        complete_count: videos.iter().filter(|video| video.complete).count(),
    }
}

fn downloaded_duration(duration: Option<u64>, bytes: u64, total: Option<u64>) -> Option<u64> {
    let duration = duration?;
    let total = total.filter(|total| *total > 0)?;
    Some(((duration as u128 * bytes as u128) / total as u128) as u64)
}

fn progress(bytes: u64, total: Option<u64>) -> Option<f64> {
    let total = total.filter(|total| *total > 0)?;
    Some((bytes as f64 / total as f64).min(1.0))
}

fn status(downloaded: u64, complete: bool) -> &'static str {
    match (complete, downloaded > 0) {
        (true, _) => "cached",
        (false, true) => "partial",
        (false, false) => "queued",
    }
}

fn range_len(range: &Range<u64>) -> u64 {
    range.end.saturating_sub(range.start)
}

fn range_snapshot(range: Range<u64>) -> RangeSnapshot {
    RangeSnapshot {
        start: range.start,
        end: range.end,
    }
}
