mod support;

use rust_lib_ghostr::video::native_models::{
    NativeVideoCacheKey, NativeVideoDelivery, NativeVideoDownload,
};
use std::path::PathBuf;
use std::time::Duration;
use support::fixtures::{event_identity, native_download, native_video, video_id};

#[tokio::test(start_paused = true)]
async fn native_download_state_tracks_retry_rejection_suppression_and_availability() {
    let mut download = native_download("https://media.example/clip.mp4");
    assert!(download.is_downloading());
    assert!(download.participates_in_cache());
    assert!(download.local_path().is_none());

    download.finish_download(None, true);
    assert!(!download.begin_retry(tokio::time::Instant::now()));
    tokio::time::advance(Duration::from_secs(1)).await;
    assert!(download.begin_retry(tokio::time::Instant::now()));
    download.finish_download(None, false);
    assert!(download.is_rejected());

    let owner = NativeVideoCacheKey::AdvertisedDigest("c".repeat(64));
    download.suppress(owner.clone());
    assert_eq!(download.suppressed_by(), Some(&owner));
    assert!(!download.is_rejected());
    download.restart_download();
    assert!(download.is_downloading());
    assert!(download.suppressed_by().is_none());

    let available = PathBuf::from("/cache/clip.mp4");
    download.mark_available(available.clone());
    assert_eq!(download.local_path(), Some(available.as_path()));
    assert!(download.suppressed_by().is_none());

    let mut hls_video = native_video("https://media.example/live.m3u8");
    hls_video.delivery = NativeVideoDelivery::Hls;
    let mut hls = NativeVideoDownload::new(video_id(), hls_video, event_identity());
    assert!(!hls.is_downloading());
    assert!(!hls.participates_in_cache());
    assert!(!hls.begin_retry(tokio::time::Instant::now()));
}
