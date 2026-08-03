mod support;

use std::path::PathBuf;
use std::time::Duration;
use support::fixtures::native_download;

#[tokio::test(start_paused = true)]
async fn retries_a_failed_refresh_of_an_available_download() {
    let mut download = native_download("https://media.example/video.mp4");
    let cached = PathBuf::from("cached-video.mp4");
    download.finish_download(Some(cached.clone()), false);
    assert_eq!(download.local_path(), Some(cached.as_path()));

    download.finish_download(None, true);
    assert!(download.local_path().is_none());
    assert!(!download.begin_retry(tokio::time::Instant::now()));
    tokio::time::advance(Duration::from_secs(1)).await;

    assert!(download.begin_retry(tokio::time::Instant::now()));
    assert!(download.is_downloading());
}
