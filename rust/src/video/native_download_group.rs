use crate::video::native_models::{NativeVideoCacheKey, NativeVideoDownload};
use std::collections::HashMap;

#[derive(Clone)]
pub struct NativeDownloadGroup {
    pub cache_key: NativeVideoCacheKey,
    pub expected_digest: Option<String>,
    pub urls: Vec<String>,
    pub videos: Vec<NativeVideoDownload>,
}

pub fn group_downloads(videos: Vec<NativeVideoDownload>) -> Vec<NativeDownloadGroup> {
    let mut positions = HashMap::<NativeVideoCacheKey, usize>::new();
    let mut groups = Vec::<NativeDownloadGroup>::new();
    for video in videos {
        let cache_key = video.nostr.cache_key();
        if let Some(index) = positions.get(&cache_key) {
            let group = &mut groups[*index];
            for url in video.pending_source_urls() {
                if !group.urls.iter().any(|existing| existing == url) {
                    group.urls.push(url.to_owned());
                }
            }
            group.videos.push(video);
            continue;
        }
        let urls = video.pending_source_urls().map(str::to_owned).collect();
        positions.insert(cache_key.clone(), groups.len());
        groups.push(NativeDownloadGroup {
            cache_key,
            expected_digest: video.nostr.expected_digest.clone(),
            urls,
            videos: vec![video],
        });
    }
    groups
}
