use crate::video::event_identity::CanonicalNativeVideo;
use crate::video::native_cache::NativeVideoCache;
use crate::video::native_models::{NativeDownloads, NativeVideoCacheKey, NativeVideoDownload};
use std::collections::HashSet;

pub async fn preempt_lower_ranked(
    downloads: &NativeDownloads,
    cache: &NativeVideoCache,
    videos: &[CanonicalNativeVideo],
    pending: &[NativeVideoDownload],
) -> anyhow::Result<()> {
    let Some(priority) = pending.first().map(|item| item.nostr.cache_key()) else {
        return Ok(());
    };
    let lower = lower_ranked_cache_keys(videos, &priority);
    let evicted = cache.preempt(&priority, &lower).await?;
    suppress_evicted(downloads, &evicted, priority).await;
    Ok(())
}

fn lower_ranked_cache_keys(
    videos: &[CanonicalNativeVideo],
    priority: &NativeVideoCacheKey,
) -> Vec<NativeVideoCacheKey> {
    let position = videos
        .iter()
        .position(|item| item.video.cache_key() == *priority)
        .expect("pending cache key belongs to native inventory");
    let mut unique = HashSet::new();
    let mut lower = videos[position + 1..]
        .iter()
        .map(|item| item.video.cache_key())
        .filter(|key| key != priority)
        .filter(|key| unique.insert(key.clone()))
        .collect::<Vec<_>>();
    lower.reverse();
    lower
}

async fn suppress_evicted(
    downloads: &NativeDownloads,
    evicted: &HashSet<NativeVideoCacheKey>,
    priority: NativeVideoCacheKey,
) {
    downloads
        .lock()
        .await
        .values_mut()
        .filter(|item| evicted.contains(&item.nostr.cache_key()))
        .for_each(|item| item.suppress(priority.clone()));
}
