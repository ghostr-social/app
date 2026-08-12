use crate::progressive::route::ProgressiveState;
use ghostr_delivery::cache_registry::CacheVideo;
use ghostr_delivery::debug::feed::DebugFeedMetadata;
use ghostr_engine::host_stats::host_of;
use serde::Serialize;
use std::ops::Range;

#[derive(Debug, Serialize)]
pub(super) struct VideoSnapshot {
    id: String,
    nostr_event_id: Option<String>,
    title: Option<String>,
    creator: Option<String>,
    created_at: Option<u64>,
    feed_index: Option<usize>,
    focus_distance: Option<i64>,
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
struct RangeSnapshot {
    start: u64,
    end: u64,
}

struct VideoFacts {
    video: CacheVideo,
    metadata: Option<DebugFeedMetadata>,
    total: Option<u64>,
    ranges: Vec<Range<u64>>,
    downloaded: u64,
    complete: bool,
    playback_url: String,
}

pub(super) async fn snapshot(state: &ProgressiveState, video: CacheVideo) -> VideoSnapshot {
    let metadata = state.debug_feed.metadata(&video.id);
    let total = total_len(state, &video).await;
    let ranges = present_ranges(state, &video.id).await;
    let downloaded = ranges.iter().map(range_len).sum();
    let complete = total.is_some_and(|size| downloaded >= size);
    let playback_url = playback_url(state, &video.id).await;
    VideoFacts {
        video,
        metadata,
        total,
        ranges,
        downloaded,
        complete,
        playback_url,
    }
    .into()
}

impl From<VideoFacts> for VideoSnapshot {
    fn from(facts: VideoFacts) -> Self {
        Self {
            nostr_event_id: facts.metadata.as_ref().map(|item| item.event_id.clone()),
            title: facts.metadata.as_ref().and_then(|item| item.title.clone()),
            creator: facts.metadata.as_ref().map(|item| item.creator.clone()),
            created_at: facts.metadata.as_ref().map(|item| item.created_at),
            feed_index: facts.metadata.as_ref().map(|item| item.feed_index),
            focus_distance: facts.metadata.as_ref().map(|item| item.focus_distance),
            source_host: facts.video.meta.urls.first().and_then(|url| host_of(url)),
            source_count: facts.video.meta.urls.len(),
            downloaded_duration_ms: downloaded_duration(&facts),
            progress: progress(facts.downloaded, facts.total),
            status: status(facts.downloaded, facts.complete),
            ranges: facts.ranges.into_iter().map(range_snapshot).collect(),
            duration_ms: facts.video.meta.duration_ms,
            downloaded_bytes: facts.downloaded,
            total_bytes: facts.total,
            complete: facts.complete,
            playback_url: facts.playback_url,
            id: facts.video.id,
        }
    }
}

impl VideoSnapshot {
    pub(super) fn total_bytes(&self) -> Option<u64> {
        self.total_bytes
    }

    pub(super) fn is_complete(&self) -> bool {
        self.complete
    }
}

async fn playback_url(state: &ProgressiveState, id: &str) -> String {
    let capability = state.capabilities.issue(id).await;
    format!("/video.mp4?id={id}&cap={}", capability.as_str())
}

async fn total_len(state: &ProgressiveState, video: &CacheVideo) -> Option<u64> {
    state
        .store
        .total_len(&video.id)
        .await
        .unwrap_or(video.meta.size_bytes)
}

async fn present_ranges(state: &ProgressiveState, id: &str) -> Vec<Range<u64>> {
    state.store.present_ranges(id).await.unwrap_or_default()
}

fn downloaded_duration(facts: &VideoFacts) -> Option<u64> {
    let duration = facts.video.meta.duration_ms?;
    let total = facts.total.filter(|total| *total > 0)?;
    Some(((duration as u128 * facts.downloaded as u128) / total as u128) as u64)
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
