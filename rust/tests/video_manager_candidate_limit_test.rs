mod support;

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::Router;
use nostr_sdk::{EventBuilder, Keys, Kind, Tag, Timestamp};
use rust_lib_ghostr::video::event_identity::canonical_native_videos;
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
use support::fixtures::temp_directory;
use tokio::sync::Mutex;

#[tokio::test]
async fn bounded_attempts_leave_untried_mirrors_eligible() {
    let calls = Arc::new(AtomicUsize::new(0));
    let app = Router::new().fallback(origin).with_state(calls.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener");
    let address = listener.local_addr().expect("address");
    let server = tokio::spawn(axum::serve(listener, app).into_future());
    let digest = format!("{:x}", Sha256::digest(b"video"));
    let videos = new_native_video_index();
    for (created_at, path) in [(30, "bad-a"), (20, "bad-b"), (10, "good")] {
        let url = format!("http://{address}/{path}");
        videos
            .insert(canonical_native_videos(&video_event(&url, &digest, created_at)).remove(0))
            .await;
    }
    let directory = temp_directory("ghostr-candidate-limit");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let downloads = new_native_downloads();
    let cache = NativeVideoCache::new(directory.clone(), 10, Arc::new(Mutex::new(0)));
    let configuration =
        NativeVideoManagerConfiguration::new(MediaHttpClient::trusted().expect("client"), 1)
            .with_candidate_policy(2, Duration::from_secs(30));
    let manager =
        NativeVideoManager::with_configuration(downloads.clone(), cache, videos, configuration);

    manager.synchronize_once().await.expect("bounded attempt");
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    manager.synchronize_once().await.expect("remaining mirror");

    assert_eq!(calls.load(Ordering::SeqCst), 3);
    assert!(downloads
        .lock()
        .await
        .values()
        .all(|item| item.local_path().is_some()));
    server.abort();
    std::fs::remove_dir_all(directory).expect("remove cache");
}

async fn origin(
    State(calls): State<Arc<AtomicUsize>>,
    request: Request,
) -> (StatusCode, &'static str) {
    calls.fetch_add(1, Ordering::SeqCst);
    if request.uri().path() == "/good" {
        (StatusCode::OK, "video")
    } else {
        (StatusCode::NOT_FOUND, "")
    }
}

fn video_event(url: &str, digest: &str, created_at: u64) -> nostr_sdk::Event {
    EventBuilder::new(Kind::Custom(22), url)
        .custom_created_at(Timestamp::from(created_at))
        .tags([Tag::parse([
            "imeta".to_owned(),
            format!("url {url}"),
            format!("x {digest}"),
            "m video/mp4".to_owned(),
        ])
        .expect("metadata")])
        .sign_with_keys(&Keys::generate())
        .expect("signed video")
}
