mod support;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Router;
use rust_lib_ghostr::video::event_index::new_native_video_index;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use rust_lib_ghostr::video::outbound_media_client::MediaHttpClient;
use rust_lib_ghostr::video::video_manager::{NativeVideoManager, NativeVideoManagerConfiguration};
use sha2::{Digest, Sha256};
use std::future::IntoFuture;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use support::fixtures::{canonical_video, temp_directory};
use tokio::sync::Mutex;

#[tokio::test]
async fn bounded_cycles_exhaust_each_candidate_before_rejecting_a_video() {
    let calls = Arc::new(AtomicUsize::new(0));
    let app = Router::new().fallback(origin).with_state(calls.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener");
    let address = listener.local_addr().expect("address");
    let server = tokio::spawn(axum::serve(listener, app).into_future());
    let sources = (0..5)
        .map(|index| format!("http://{address}/bad-{index}"))
        .collect::<Vec<_>>();
    let mut video = canonical_video(&sources[0]);
    video.video.expected_digest = Some(format!("{:x}", Sha256::digest(b"video")));
    video.video.fallback_urls = sources[1..].to_vec();
    let videos = new_native_video_index();
    videos.insert(video).await;
    let directory = temp_directory("ghostr-candidate-exhaustion");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let downloads = new_native_downloads();
    let cache = NativeVideoCache::new(directory.clone(), 10, Arc::new(Mutex::new(0)));
    let configuration =
        NativeVideoManagerConfiguration::new(MediaHttpClient::trusted().expect("client"), 1)
            .with_candidate_policy(2, Duration::from_secs(30));
    let manager =
        NativeVideoManager::with_configuration(downloads.clone(), cache, videos, configuration);

    for expected_calls in [2, 4, 5] {
        manager.synchronize_once().await.expect("candidate cycle");
        assert_eq!(calls.load(Ordering::SeqCst), expected_calls);
    }
    assert!(downloads
        .lock()
        .await
        .values()
        .all(|item| item.is_rejected()));
    manager.synchronize_once().await.expect("terminal cycle");
    assert_eq!(calls.load(Ordering::SeqCst), 5);

    server.abort();
    std::fs::remove_dir_all(directory).expect("remove cache");
}

async fn origin(State(calls): State<Arc<AtomicUsize>>) -> (StatusCode, &'static str) {
    calls.fetch_add(1, Ordering::SeqCst);
    (StatusCode::OK, "evil!")
}
