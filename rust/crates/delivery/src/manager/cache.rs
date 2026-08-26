//! Projects playable candidates and stored-byte state into the cache registry.

use crate::cache_registry::{CacheStatus, CacheVideo};
use crate::manager::DeliveryWorker;
use core::ops::Range;
use ghostr_engine::{PostId, VideoMeta};

impl DeliveryWorker {
    pub(super) async fn refresh_cache_registry(&self, observed_at_ms: u64) {
        let mut videos = Vec::new();
        let mut blocked = Vec::new();
        for post in self.state.candidate_posts() {
            if let Some(video) = self.cache_video(post.clone()).await {
                if self
                    .state
                    .planner_capability(&post, observed_at_ms)
                    .blocks_direct_playback()
                {
                    blocked.push(post.as_str().to_owned());
                }
                videos.push(video);
            }
        }
        self.cache.replace_with_blocked(videos, blocked);
    }

    pub(super) fn is_servable(&self, post: &PostId) -> bool {
        self.state
            .catalog()
            .lookup(post)
            .is_some_and(|entry| !self.retry.all_retired(post, &entry.meta.urls))
    }

    async fn cache_video(&self, post: PostId) -> Option<CacheVideo> {
        if !self.is_servable(&post) {
            return None;
        }
        let entry = self.state.catalog().lookup(&post)?;
        let snapshot = self.ctx.store.media_snapshot(post.as_str()).await.ok()?;
        if !snapshot
            .binding()
            .is_some_and(|binding| binding.matches_or_derives_from(&entry.meta))
        {
            return None;
        }
        Some(cached(
            post,
            entry.meta.clone(),
            snapshot.is_complete(),
            snapshot.ranges(),
        ))
    }
}

fn cached(post: PostId, meta: VideoMeta, complete: bool, ranges: &[Range<u64>]) -> CacheVideo {
    let downloaded: u64 = ranges.iter().map(range_len).sum();
    CacheVideo {
        id: post.0,
        meta,
        status: cache_status(downloaded, complete),
    }
}

fn cache_status(downloaded: u64, complete: bool) -> CacheStatus {
    if complete {
        CacheStatus::Complete
    } else if downloaded > 0 {
        CacheStatus::Partial
    } else {
        CacheStatus::Ready
    }
}

fn range_len(range: &Range<u64>) -> u64 {
    range.end.saturating_sub(range.start)
}
