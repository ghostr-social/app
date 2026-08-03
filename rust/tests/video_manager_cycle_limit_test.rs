mod support;

use axum::extract::State;
use axum::Router;
use nostr_sdk::{EventBuilder, Keys, Kind, Tag, Timestamp};
use rust_lib_ghostr::video::event_index::{index_event, NativeVideoIndex};
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use std::future::IntoFuture;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use support::fixtures::{temp_directory, trusted_video_manager};
use tokio::sync::Mutex;

#[tokio::test]
async fn limits_each_synchronization_to_one_parallel_download_batch() {
    let calls = Arc::new(AtomicUsize::new(0));
    let app = Router::new().fallback(origin).with_state(calls.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener");
    let address = listener.local_addr().expect("address");
    let server = tokio::spawn(axum::serve(listener, app).into_future());
    let videos = NativeVideoIndex::new(10);
    for (created_at, path) in [(30, "first"), (20, "second"), (10, "third")] {
        index_event(
            &video_event(&format!("http://{address}/{path}"), created_at),
            &videos,
        )
        .await;
    }
    let directory = temp_directory("ghostr-cycle-limit");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let downloads = new_native_downloads();
    let cache = NativeVideoCache::new(directory.clone(), 100, Arc::new(Mutex::new(0)));
    let manager = trusted_video_manager(downloads.clone(), cache, videos, 2);

    manager.synchronize_once().await.expect("first cycle");
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    manager.synchronize_once().await.expect("second cycle");
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    assert!(downloads
        .lock()
        .await
        .values()
        .all(|item| item.local_path().is_some()));
    server.abort();
    std::fs::remove_dir_all(directory).expect("remove cache");
}

async fn origin(State(calls): State<Arc<AtomicUsize>>) -> &'static str {
    calls.fetch_add(1, Ordering::SeqCst);
    "video"
}

fn video_event(url: &str, created_at: u64) -> nostr_sdk::Event {
    EventBuilder::new(Kind::Custom(22), url)
        .custom_created_at(Timestamp::from(created_at))
        .tags([Tag::parse([
            "imeta".to_owned(),
            format!("url {url}"),
            "m video/mp4".to_owned(),
        ])
        .expect("metadata")])
        .sign_with_keys(&Keys::generate())
        .expect("signed video")
}
