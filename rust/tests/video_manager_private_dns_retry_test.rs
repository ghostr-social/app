mod support;

use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use rust_lib_ghostr::video::event_index::new_native_video_index;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use rust_lib_ghostr::video::outbound_media_client::MediaHttpClient;
use rust_lib_ghostr::video::video_manager::{NativeVideoManager, NativeVideoManagerConfiguration};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use support::fixtures::{canonical_video, temp_directory};
use tokio::sync::Mutex;

struct CountingPrivateResolver(Arc<AtomicUsize>);

impl Resolve for CountingPrivateResolver {
    fn resolve(&self, _name: Name) -> Resolving {
        self.0.fetch_add(1, Ordering::SeqCst);
        let private = SocketAddr::from((Ipv4Addr::LOCALHOST, 80));
        let addresses: Addrs = Box::new(vec![private].into_iter());
        Box::pin(async move { Ok(addresses) })
    }
}

#[tokio::test(start_paused = true)]
async fn does_not_retry_a_hostname_rejected_by_the_address_policy() {
    let calls = Arc::new(AtomicUsize::new(0));
    let client = MediaHttpClient::with_resolver(Arc::new(CountingPrivateResolver(calls.clone())))
        .expect("media client");
    let directory = temp_directory("ghostr-private-dns-retry");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = new_native_video_index();
    videos
        .insert(canonical_video("http://media.test/video.mp4"))
        .await;
    let downloads = new_native_downloads();
    let cache = NativeVideoCache::new(directory.clone(), 10, Arc::new(Mutex::new(0)));
    let configuration = NativeVideoManagerConfiguration::new(client, 1);
    let manager = NativeVideoManager::with_configuration(downloads, cache, videos, configuration);

    manager.synchronize_once().await.expect("first attempt");
    let calls_after_rejection = calls.load(Ordering::SeqCst);
    assert!(calls_after_rejection > 0);
    tokio::time::advance(Duration::from_secs(1)).await;
    manager.synchronize_once().await.expect("retry boundary");

    assert_eq!(calls.load(Ordering::SeqCst), calls_after_rejection);
    std::fs::remove_dir_all(directory).expect("remove cache");
}
