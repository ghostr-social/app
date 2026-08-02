use crate::video::event_identity::CanonicalNativeVideo;
use crate::video::event_index::NativeVideoIndex;
use crate::video::native_cache::NativeVideoCache;
use crate::video::native_models::{NativeDownloads, NativeVideoDownload};
use log::warn;
use reqwest::Client;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

pub struct NativeVideoManager {
    downloads: NativeDownloads,
    cache: Arc<NativeVideoCache>,
    client: Client,
    max_parallel_downloads: usize,
    videos: NativeVideoIndex,
}

struct NativeDownloadGroup {
    cache_id: String,
    url: String,
    videos: Vec<NativeVideoDownload>,
}

impl NativeVideoManager {
    pub fn new(
        downloads: NativeDownloads,
        cache: NativeVideoCache,
        videos: NativeVideoIndex,
        max_parallel_downloads: usize,
    ) -> Self {
        Self {
            downloads,
            cache: Arc::new(cache),
            client: Client::new(),
            max_parallel_downloads: max_parallel_downloads.max(1),
            videos,
        }
    }

    pub fn start(self) {
        tokio::spawn(self.run());
    }

    async fn run(self) {
        loop {
            if let Err(error) = self.synchronize_once().await {
                warn!("Native video synchronization failed: {error}");
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    pub async fn synchronize_once(&self) -> anyhow::Result<()> {
        let videos = self.videos.ordered_videos().await;
        let pending = self.pending_downloads(videos).await;
        self.download(group_downloads(pending)).await
    }

    async fn pending_downloads(
        &self,
        videos: Vec<CanonicalNativeVideo>,
    ) -> Vec<NativeVideoDownload> {
        let active = videos
            .iter()
            .map(|item| item.inventory_id.clone())
            .collect::<HashSet<_>>();
        let mut downloads = self.downloads.lock().await;
        downloads.retain(|id, _| active.contains(id));
        videos
            .into_iter()
            .filter_map(|item| insert_pending(&mut downloads, item))
            .collect()
    }

    async fn download(&self, groups: Vec<NativeDownloadGroup>) -> anyhow::Result<()> {
        for batch in groups.chunks(self.max_parallel_downloads) {
            let tasks = batch
                .iter()
                .cloned()
                .map(|group| self.spawn_download(group))
                .collect::<Vec<_>>();
            for task in tasks {
                task.await?;
            }
        }
        Ok(())
    }

    fn spawn_download(&self, group: NativeDownloadGroup) -> tokio::task::JoinHandle<()> {
        let downloads = self.downloads.clone();
        let cache = self.cache.clone();
        let client = self.client.clone();
        tokio::spawn(async move {
            let result = cache.download(&client, &group.cache_id, &group.url).await;
            let path = match result {
                Ok(cached) => Some(cached.path),
                Err(error) => {
                    warn!("Native video cache skipped {}: {error}", group.url);
                    None
                }
            };
            update_downloads(downloads, group.videos, path).await;
        })
    }
}

impl Clone for NativeDownloadGroup {
    fn clone(&self) -> Self {
        Self {
            cache_id: self.cache_id.clone(),
            url: self.url.clone(),
            videos: self.videos.clone(),
        }
    }
}

fn insert_pending(
    downloads: &mut HashMap<String, NativeVideoDownload>,
    item: CanonicalNativeVideo,
) -> Option<NativeVideoDownload> {
    if downloads.contains_key(&item.inventory_id) {
        return None;
    }
    let download = NativeVideoDownload::new(item.inventory_id, item.video, item.identity);
    downloads.insert(download.id.clone(), download.clone());
    download
        .nostr
        .delivery
        .can_cache_as_single_file()
        .then_some(download)
}

fn group_downloads(videos: Vec<NativeVideoDownload>) -> Vec<NativeDownloadGroup> {
    let mut positions = HashMap::<String, usize>::new();
    let mut groups = Vec::<NativeDownloadGroup>::new();
    for video in videos {
        let cache_id = video.nostr.id.clone();
        if let Some(index) = positions.get(&cache_id) {
            groups[*index].videos.push(video);
            continue;
        }
        positions.insert(cache_id.clone(), groups.len());
        groups.push(NativeDownloadGroup {
            cache_id,
            url: video.url.clone(),
            videos: vec![video],
        });
    }
    groups
}

async fn update_downloads(
    downloads: NativeDownloads,
    videos: Vec<NativeVideoDownload>,
    path: Option<PathBuf>,
) {
    let mut downloads = downloads.lock().await;
    for video in videos {
        let Some(current) = downloads.get_mut(&video.id) else {
            continue;
        };
        current.downloading = false;
        current.local_path = path.clone();
    }
}
