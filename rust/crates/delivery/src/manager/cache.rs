//! Projects playable candidates and stored-byte state into the cache registry.

use crate::cache_registry::{CacheStatus, CacheVideo};
use crate::manager::DeliveryWorker;
use ghostr_engine::{PostId, VideoMeta};
use std::ops::Range;

impl DeliveryWorker {
    pub(crate) async fn refresh_cache_registry(&self) {
        let mut videos = Vec::new();
        for post in self.state.candidate_posts() {
            if let Some(video) = self.cache_video(post).await {
                videos.push(video);
            }
        }
        self.cache.replace(videos);
    }

    pub(crate) fn is_servable(&self, post: &PostId) -> bool {
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
