use crate::video::event_identity::CanonicalNativeVideo;
use crate::video::native_download_candidates::{NativeCandidateFailure, NativeDownloadOutcome};
use crate::video::native_download_group::NativeDownloadGroup;
use crate::video::native_models::{NativeDownloads, NativeVideoCacheKey, NativeVideoDownload};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub fn insert_pending(
    downloads: &mut HashMap<String, NativeVideoDownload>,
    item: CanonicalNativeVideo,
    invalid: &HashSet<NativeVideoCacheKey>,
) -> Option<NativeVideoDownload> {
    if let Some(existing) = downloads.get_mut(&item.inventory_id) {
        if existing.participates_in_cache() && invalid.contains(&existing.nostr.cache_key()) {
            existing.restart_download();
            return Some(existing.clone());
        }
        let due = existing.begin_retry(tokio::time::Instant::now());
        return (due || existing.is_downloading()).then(|| existing.clone());
    }
    insert_new(downloads, item)
}

fn insert_new(
    downloads: &mut HashMap<String, NativeVideoDownload>,
    item: CanonicalNativeVideo,
) -> Option<NativeVideoDownload> {
    let download = NativeVideoDownload::new(item.inventory_id, item.video, item.identity);
    downloads.insert(download.id.clone(), download.clone());
    download
        .nostr
        .delivery
        .can_cache_as_single_file()
        .then_some(download)
}

pub async fn apply_group_outcome(
    downloads: NativeDownloads,
    group: NativeDownloadGroup,
    outcome: NativeDownloadOutcome,
) {
    let mut downloads = downloads.lock().await;
    if let Some(path) = outcome.path {
        promote_shared_blob(&mut downloads, &group.cache_key, path);
        return;
    }
    apply_failures(&mut downloads, &group.videos, &outcome.failures);
}

fn promote_shared_blob(
    downloads: &mut HashMap<String, NativeVideoDownload>,
    key: &NativeVideoCacheKey,
    path: PathBuf,
) {
    downloads
        .values_mut()
        .filter(|item| item.nostr.cache_key() == *key)
        .filter(|item| item.nostr.delivery.can_cache_as_single_file())
        .for_each(|item| item.mark_available(path.clone()));
}

fn apply_failures(
    downloads: &mut HashMap<String, NativeVideoDownload>,
    videos: &[NativeVideoDownload],
    failures: &[NativeCandidateFailure],
) {
    for video in videos {
        let Some(current) = downloads.get_mut(&video.id) else {
            continue;
        };
        let mut matched = false;
        for failure in failures {
            matched |= current.record_candidate_failure(&failure.url, failure.retryable);
        }
        if matched {
            current.finish_candidate_round_if_exhausted();
        }
    }
}
