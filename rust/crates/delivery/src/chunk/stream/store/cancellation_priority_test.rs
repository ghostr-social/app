use super::pace_or_cancel;
use crate::chunk::cancel::cancel_pair;
use crate::debug::network::NetworkThrottle;

#[tokio::test]
async fn ready_cancellation_always_wins_over_inert_pacing() {
    let throttle = NetworkThrottle::new();
    for _ in 0..64 {
        let (handle, token) = cancel_pair();
        handle.cancel();

        assert!(pace_or_cancel(Some(&throttle), 1, &token).await);
    }
}

#[tokio::test]
async fn ready_cancellation_stops_an_unpaced_write() {
    let (handle, token) = cancel_pair();
    handle.cancel();

    assert!(pace_or_cancel(None, 1, &token).await);
}
