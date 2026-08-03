mod support;

use rust_lib_ghostr::video::event_index::NativeVideoIndex;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::{new_native_downloads, NativeVideoDelivery};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use support::fixtures::{canonical_video, temp_directory, trusted_video_manager};
use support::http::{spawn_raw_server, spawn_response_sequence};
use tokio::sync::Mutex;

#[tokio::test]
async fn hls_row_does_not_keep_a_lower_progressive_row_suppressed() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo";
    let priority_response =
        b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nfresh";
    let (lower_url, lower_requests) = spawn_response_sequence(vec![response, response]).await;
    let (priority_url, priority_request) = spawn_raw_server(priority_response).await;
    let directory = temp_directory("ghostr-hls-cache-claim");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = NativeVideoIndex::new(3);
    videos.insert(canonical_video(&lower_url)).await;
    let downloads = new_native_downloads();
    let used_bytes = Arc::new(Mutex::new(0));
    let cache = NativeVideoCache::new(directory.clone(), 5, used_bytes.clone());
    let manager = trusted_video_manager(downloads.clone(), cache, videos.clone(), 1);
    manager.synchronize_once().await.expect("cache lower row");
    let digest = format!("{:x}", Sha256::digest(b"fresh"));
    videos
        .insert(video(
            &priority_url,
            &digest,
            NativeVideoDelivery::Progressive,
            ("b", "priority-event", 43),
        ))
        .await;
    manager
        .synchronize_once()
        .await
        .expect("cache priority row");

    videos
        .insert(video(
            "https://media.example/live.m3u8",
            &digest,
            NativeVideoDelivery::Hls,
            ("c", "hls-event", 44),
        ))
        .await;
    manager.synchronize_once().await.expect("register HLS row");
    videos
        .insert(video(
            "https://media.example/replacement.m3u8",
            &"d".repeat(64),
            NativeVideoDelivery::Hls,
            ("d", "priority-event", 45),
        ))
        .await;
    manager
        .synchronize_once()
        .await
        .expect("remove priority row");

    let values = downloads.lock().await;
    assert!(values[&"a".repeat(64)].local_path().is_some());
    assert_eq!(*used_bytes.lock().await, 5);
    drop(values);
    lower_requests.await.expect("lower row retried");
    priority_request.await.expect("priority request");
    std::fs::remove_dir_all(directory).expect("remove cache");
}

fn video(
    url: &str,
    digest: &str,
    delivery: NativeVideoDelivery,
    identity: (&str, &str, u64),
) -> rust_lib_ghostr::video::event_identity::CanonicalNativeVideo {
    let mut video = canonical_video(url);
    video.inventory_id = identity.0.repeat(64);
    video.coordinate = identity.1.to_owned();
    video.identity.event_id = video.inventory_id.clone();
    video.identity.created_at = identity.2;
    video.video.id = digest.to_owned();
    video.video.expected_digest = Some(digest.to_owned());
    video.video.delivery = delivery;
    video
}
