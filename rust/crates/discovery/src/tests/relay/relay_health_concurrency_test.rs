use crate::relay::health::RelayHealth;
use std::time::Duration;

const RELAY: &str = "wss://failed.example";

#[tokio::test(start_paused = true)]
async fn concurrent_normal_failures_open_one_initial_backoff() {
    let health = RelayHealth::new();
    let first = health.admit(&urls());
    let second = health.admit(&urls());

    health.observe(&first, &[], &[RELAY.to_owned()]);
    health.observe(&second, &[], &[RELAY.to_owned()]);
    tokio::time::advance(Duration::from_secs(2)).await;

    let recovery = health.admit(&urls());
    assert_eq!(recovery.len(), 1, "one outage must retain initial backoff");
    health.observe(&recovery, &[], &[RELAY.to_owned()]);
    tokio::time::advance(Duration::from_secs(3)).await;
    assert!(health.admit(&urls()).is_empty());
    tokio::time::advance(Duration::from_secs(1)).await;
    assert_eq!(health.admit(&urls()).len(), 1);
}

#[tokio::test(start_paused = true)]
async fn older_normal_failure_cannot_revoke_a_live_recovery_probe() {
    let health = RelayHealth::new();
    let first = health.admit(&urls());
    let second = health.admit(&urls());
    health.observe(&first, &[], &[RELAY.to_owned()]);
    tokio::time::advance(Duration::from_secs(2)).await;
    let probe = health.admit(&urls());

    health.observe(&second, &[], &[RELAY.to_owned()]);
    tokio::time::advance(Duration::from_secs(4)).await;

    assert!(health.admit(&urls()).is_empty(), "probe lease was revoked");
    health.release(&probe);
    assert!(health.admit(&urls()).is_empty());
    tokio::time::advance(Duration::from_secs(2)).await;
    assert_eq!(health.admit(&urls()).len(), 1);
}

#[tokio::test(start_paused = true)]
async fn cancelled_recovery_probes_cannot_storm_without_elapsed_backoff() {
    let health = RelayHealth::new();
    let initial = health.admit(&urls());
    health.observe(&initial, &[], &[RELAY.to_owned()]);
    tokio::time::advance(Duration::from_secs(2)).await;

    for _ in 0..3 {
        let probe = health.admit(&urls());
        assert_eq!(probe.len(), 1);
        health.release(&probe);
        assert!(health.admit(&urls()).is_empty());
        tokio::time::advance(Duration::from_secs(1)).await;
        assert!(health.admit(&urls()).is_empty());
        tokio::time::advance(Duration::from_secs(1)).await;
    }
}

fn urls() -> Vec<String> {
    vec![RELAY.to_owned()]
}
