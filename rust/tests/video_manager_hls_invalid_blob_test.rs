mod support;

use rust_lib_ghostr::video::event_index::NativeVideoIndex;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::{new_native_downloads, NativeVideoDelivery};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use support::fixtures::{canonical_video, temp_directory, trusted_video_manager};
use support::http::{spawn_response_sequence, unused_loopback_url};
use tokio::sync::Mutex;

#[tokio::test]
async fn invalid_shared_blob_does_not_restart_an_hls_row() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo";
    let (progressive_url, requests) = spawn_response_sequence(vec![response, response]).await;
    let directory = temp_directory("ghostr-hls-invalid-blob");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let digest = format!("{:x}", Sha256::digest(b"video"));
    let videos = NativeVideoIndex::new(2);
    videos
        .insert(row(
            &progressive_url,
            &digest,
            NativeVideoDelivery::Progressive,
            ("a", 42),
        ))
        .await;
    videos
        .insert(row(
            &unused_loopback_url().await,
            &digest,
            NativeVideoDelivery::Hls,
            ("b", 43),
        ))
        .await;
    let downloads = new_native_downloads();
    let cache = NativeVideoCache::new(directory.clone(), 10, Arc::new(Mutex::new(0)));
    let manager = trusted_video_manager(downloads.clone(), cache, videos, 1);
    manager
        .synchronize_once()
        .await
        .expect("cache progressive row");
    let path = downloads.lock().await[&"a".repeat(64)]
        .local_path()
        .expect("cached path")
        .to_path_buf();
    tokio::fs::remove_file(path).await.expect("remove blob");

    manager
        .synchronize_once()
        .await
        .expect("repair invalid blob");

    let values = downloads.lock().await;
    assert!(!values[&"b".repeat(64)].is_downloading());
    assert!(values[&"b".repeat(64)].local_path().is_none());
    drop(values);
    requests.await.expect("progressive row retried");
    std::fs::remove_dir_all(directory).expect("remove cache");
}

fn row(
    url: &str,
    digest: &str,
    delivery: NativeVideoDelivery,
    identity: (&str, u64),
) -> rust_lib_ghostr::video::event_identity::CanonicalNativeVideo {
    let mut row = canonical_video(url);
    row.inventory_id = identity.0.repeat(64);
    row.coordinate = row.inventory_id.clone();
    row.identity.event_id = row.inventory_id.clone();
    row.identity.created_at = identity.1;
    row.video.id = digest.to_owned();
    row.video.expected_digest = Some(digest.to_owned());
    row.video.delivery = delivery;
    row
}
