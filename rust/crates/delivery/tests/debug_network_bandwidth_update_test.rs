use ghostr_delivery::debug::network::{NetworkProfile, NetworkThrottle};
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn a_live_rate_change_reprices_only_the_unserved_bytes() {
    let throttle = NetworkThrottle::new();
    throttle.update(profile(8));
    let transfer = tokio::spawn({
        let throttle = throttle.clone();
        async move { throttle.pace(1_000).await }
    });
    tokio::task::yield_now().await;

    tokio::time::advance(Duration::from_millis(500)).await;
    throttle.update(profile(16));
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(249)).await;
    tokio::task::yield_now().await;
    assert!(!transfer.is_finished());

    tokio::time::advance(Duration::from_millis(1)).await;
    tokio::task::yield_now().await;
    assert!(transfer.is_finished());
    transfer.await.unwrap();
}

fn profile(bandwidth_kbps: u64) -> NetworkProfile {
    NetworkProfile {
        bandwidth_kbps,
        ..NetworkProfile::default()
    }
}
