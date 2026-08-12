use ghostr_delivery::debug::network::{NetworkProfile, NetworkThrottle};
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn canceling_the_link_owner_leaves_no_ghost_bandwidth_debt() {
    let throttle = configured_throttle();
    let abandoned = tokio::spawn({
        let throttle = throttle.clone();
        async move { throttle.pace(2_000).await }
    });
    tokio::task::yield_now().await;
    let queued = tokio::spawn(async move {
        throttle.pace(1_000).await;
    });
    tokio::task::yield_now().await;

    tokio::time::advance(Duration::from_millis(500)).await;
    abandoned.abort();
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(999)).await;
    tokio::task::yield_now().await;
    assert!(!queued.is_finished());

    tokio::time::advance(Duration::from_millis(1)).await;
    tokio::task::yield_now().await;
    queued.await.unwrap();
}

fn configured_throttle() -> NetworkThrottle {
    let throttle = NetworkThrottle::new();
    throttle.update(NetworkProfile {
        bandwidth_kbps: 8,
        ..NetworkProfile::default()
    });
    throttle
}
