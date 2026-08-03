use crate::video::event_identity::CanonicalNativeVideo;
use crate::video::event_index::NativeVideoIndex;
use crate::video::native_cache::NativeVideoCache;
use crate::video::native_cache_priority::preempt_lower_ranked;
use crate::video::native_download_candidates::{download_candidates, NativeCandidatePolicy};
use crate::video::native_download_group::{group_downloads, NativeDownloadGroup};
use crate::video::native_download_updates::{apply_group_outcome, insert_pending};
use crate::video::native_models::{NativeDownloads, NativeVideoCacheKey, NativeVideoDownload};
use crate::video::outbound_media_client::MediaHttpClient;
use log::warn;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

pub struct NativeVideoManager {
    downloads: NativeDownloads,
    cache: Arc<NativeVideoCache>,
    client: MediaHttpClient,
    max_parallel_downloads: usize,
    candidate_policy: NativeCandidatePolicy,
    videos: NativeVideoIndex,
}

pub struct NativeVideoManagerConfiguration {
    pub client: MediaHttpClient,
    pub max_parallel_downloads: usize,
    candidate_policy: NativeCandidatePolicy,
}

impl NativeVideoManagerConfiguration {
    pub fn new(client: MediaHttpClient, max_parallel_downloads: usize) -> Self {
        Self {
            client,
            max_parallel_downloads,
            candidate_policy: NativeCandidatePolicy::default(),
        }
    }

    pub fn with_candidate_policy(mut self, limit: usize, timeout: Duration) -> Self {
        self.candidate_policy = NativeCandidatePolicy::new(limit, timeout);
        self
    }
}

impl NativeVideoManager {
    pub fn new(
        downloads: NativeDownloads,
        cache: NativeVideoCache,
        videos: NativeVideoIndex,
        max_parallel_downloads: usize,
    ) -> anyhow::Result<Self> {
        let configuration = NativeVideoManagerConfiguration::new(
            MediaHttpClient::public()?,
            max_parallel_downloads,
        );
        Ok(Self::with_configuration(
            downloads,
            cache,
            videos,
            configuration,
        ))
    }

    pub fn with_configuration(
        downloads: NativeDownloads,
        cache: NativeVideoCache,
        videos: NativeVideoIndex,
        configuration: NativeVideoManagerConfiguration,
    ) -> Self {
        Self {
            downloads,
            cache: Arc::new(cache),
            client: configuration.client,
            max_parallel_downloads: configuration.max_parallel_downloads.max(1),
            candidate_policy: configuration.candidate_policy,
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
        let active_blobs = videos
            .iter()
            .filter(|item| item.video.delivery.can_cache_as_single_file())
            .map(|item| item.video.cache_key())
            .collect::<HashSet<_>>();
        let invalid = self.cache.retain(&active_blobs).await?;
        let pending = self.pending_downloads(videos.clone(), &invalid).await;
        preempt_lower_ranked(&self.downloads, &self.cache, &videos, &pending).await?;
        self.download(group_downloads(pending)).await
    }

    async fn pending_downloads(
        &self,
        videos: Vec<CanonicalNativeVideo>,
        invalid: &HashSet<NativeVideoCacheKey>,
    ) -> Vec<NativeVideoDownload> {
        let active = videos
            .iter()
            .map(|item| item.inventory_id.clone())
            .collect::<HashSet<_>>();
        let mut downloads = self.downloads.lock().await;
        downloads.retain(|id, _| active.contains(id));
        release_unclaimed_suppressions(&mut downloads);
        videos
            .into_iter()
            .filter_map(|item| insert_pending(&mut downloads, item, invalid))
            .collect()
    }

    async fn download(&self, groups: Vec<NativeDownloadGroup>) -> anyhow::Result<()> {
        let tasks = groups
            .into_iter()
            .take(self.max_parallel_downloads)
            .map(|group| self.spawn_download(group))
            .collect::<Vec<_>>();
        for task in tasks {
            task.await?;
        }
        Ok(())
    }

    fn spawn_download(&self, group: NativeDownloadGroup) -> tokio::task::JoinHandle<()> {
        let downloads = self.downloads.clone();
        let cache = self.cache.clone();
        let client = self.client.clone();
        let request = NativeDownloadRequest {
            group,
            policy: self.candidate_policy,
        };
        tokio::spawn(download_group(downloads, cache, client, request))
    }
}

fn release_unclaimed_suppressions(
    downloads: &mut std::collections::HashMap<String, NativeVideoDownload>,
) {
    let claims = downloads
        .values()
        .filter(|item| item.participates_in_cache() && !item.is_rejected())
        .map(|item| item.nostr.cache_key())
        .collect::<HashSet<_>>();
    downloads.values_mut().for_each(|item| {
        if item
            .suppressed_by()
            .is_some_and(|key| !claims.contains(key))
        {
            item.restart_download();
        }
    });
}

struct NativeDownloadRequest {
    group: NativeDownloadGroup,
    policy: NativeCandidatePolicy,
}

async fn download_group(
    downloads: NativeDownloads,
    cache: Arc<NativeVideoCache>,
    client: MediaHttpClient,
    request: NativeDownloadRequest,
) {
    let outcome = download_candidates(&cache, &client, &request.group, request.policy).await;
    apply_group_outcome(downloads, request.group, outcome).await;
}
