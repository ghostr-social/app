use core::time::Duration;

pub async fn wait_for_focus(cache: &ghostr_delivery::cache_registry::CacheRegistry) {
    let waiting = async {
        while !cache.contains("post") {
            cache.notifier().notified().await;
        }
    };
    tokio::time::timeout(Duration::from_secs(2), waiting)
        .await
        .expect("valid test fixture");
}
