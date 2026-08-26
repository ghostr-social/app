use core::time::Duration;
use ghostr_delivery::debug::network::{NetworkProfile, NetworkThrottle};

const QUANTUM: u64 = 16 * 1_024;
const QUANTUM_TIME: Duration = Duration::from_millis(1_024);

#[tokio::test(start_paused = true)]
async fn concurrent_transfers_share_one_fifo_bandwidth_budget() {
    let throttle = configured_throttle();
    let (events, mut received) = tokio::sync::mpsc::unbounded_channel();
    let first = paced_pair(throttle.clone(), "first", events.clone());
    tokio::task::yield_now().await;
    let second = paced_pair(throttle, "second", events);

    advance(QUANTUM_TIME).await;
    assert!(!first.is_finished());
    assert!(!second.is_finished());
    advance(QUANTUM_TIME).await;
    let first_owner = received.try_recv().expect("valid test fixture");
    let second_owner = received.try_recv().expect("valid test fixture");
    assert_ne!(
        first_owner, second_owner,
        "each waiter receives one quantum per round"
    );
    assert!(!first.is_finished());
    assert!(!second.is_finished());
    advance(QUANTUM_TIME * 2).await;

    first.await.expect("valid test fixture");
    second.await.expect("valid test fixture");
}

fn configured_throttle() -> NetworkThrottle {
    let throttle = NetworkThrottle::new();
    throttle.update(NetworkProfile {
        bandwidth_kbps: 128,
        ..NetworkProfile::default()
    });
    throttle
}

fn paced_pair(
    throttle: NetworkThrottle,
    owner: &'static str,
    events: tokio::sync::mpsc::UnboundedSender<&'static str>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        throttle.pace(QUANTUM).await;
        events.send(owner).expect("valid test fixture");
        throttle.pace(QUANTUM).await;
        events.send(owner).expect("valid test fixture");
    })
}

async fn advance(duration: Duration) {
    tokio::time::advance(duration).await;
    tokio::task::yield_now().await;
}
