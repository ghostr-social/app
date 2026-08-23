use crate::relay::health::RelayHealth;
use std::time::Duration;

const HEALTHY: &str = "wss://healthy.example";
const FAILING: &str = "wss://failing.example";

#[tokio::test(start_paused = true)]
async fn failed_relay_cools_then_allows_only_one_recovery_probe() {
    let health = RelayHealth::new();
    let first = health.admit(&urls([HEALTHY, FAILING]));
    health.observe(&first, &[HEALTHY.to_owned()], &[FAILING.to_owned()]);

    assert_eq!(admitted(&health, &[HEALTHY, FAILING]), vec![HEALTHY]);
    tokio::time::advance(Duration::from_secs(2)).await;
    let recovery = health.admit(&urls([HEALTHY, FAILING]));
    assert_eq!(relay_urls(&recovery), vec![HEALTHY, FAILING]);
    assert_eq!(admitted(&health, &[HEALTHY, FAILING]), vec![HEALTHY]);

    health.release(&recovery);
    assert_eq!(admitted(&health, &[HEALTHY, FAILING]), vec![HEALTHY]);
    tokio::time::advance(Duration::from_secs(2)).await;
    assert_eq!(
        admitted(&health, &[HEALTHY, FAILING]),
        vec![HEALTHY, FAILING]
    );
}

#[tokio::test(start_paused = true)]
async fn newer_success_fences_a_late_older_failure() {
    let health = RelayHealth::new();
    let older = health.admit(&urls([FAILING]));
    let newer = health.admit(&urls([FAILING]));

    health.observe(&newer, &[FAILING.to_owned()], &[]);
    health.observe(&older, &[], &[FAILING.to_owned()]);

    assert_eq!(admitted(&health, &[FAILING]), vec![FAILING]);
}

#[tokio::test(start_paused = true)]
async fn failed_recovery_uses_exponential_backoff_and_clear_resets_it() {
    let health = RelayHealth::new();
    let first = health.admit(&urls([FAILING]));
    health.observe(&first, &[], &[FAILING.to_owned()]);
    assert!(health.admit(&urls([FAILING])).is_empty());
    tokio::time::advance(Duration::from_secs(2)).await;

    let recovery = health.admit(&urls([FAILING]));
    health.observe(&recovery, &[], &[FAILING.to_owned()]);
    tokio::time::advance(Duration::from_secs(3)).await;
    assert!(health.admit(&urls([FAILING])).is_empty());
    tokio::time::advance(Duration::from_secs(1)).await;
    assert_eq!(admitted(&health, &[FAILING]), vec![FAILING]);

    health.clear();
    assert_eq!(admitted(&health, &[FAILING]), vec![FAILING]);
}

#[tokio::test(start_paused = true)]
async fn recovery_probes_are_capped_per_batch_and_globally() {
    let health = RelayHealth::new();
    let failures = urls([
        "wss://failed-0.example",
        "wss://failed-1.example",
        "wss://failed-2.example",
        "wss://failed-3.example",
        "wss://failed-4.example",
    ]);
    let initial = health.admit(&failures);
    health.observe(&initial, &[], &failures);
    tokio::time::advance(Duration::from_secs(2)).await;

    let mut probes = vec![health.admit(&failures)];
    assert_eq!(probes[0].len(), 1, "one recovery per batch");
    for relay in failures.iter().skip(1).take(3) {
        probes.push(health.admit(std::slice::from_ref(relay)));
    }
    assert!(health.admit(&failures[4..]).is_empty(), "global cap");

    health.release(&probes.remove(0));
    assert_eq!(health.admit(&failures[4..]).len(), 1);
}

fn admitted(health: &RelayHealth, candidates: &[&str]) -> Vec<String> {
    let candidates: Vec<String> = candidates.iter().map(|value| (*value).to_owned()).collect();
    let admissions = health.admit(&candidates);
    relay_urls(&admissions)
}

fn relay_urls(admissions: &[crate::relay::health::RelayAdmission]) -> Vec<String> {
    admissions
        .iter()
        .map(|item| item.url().to_owned())
        .collect()
}

fn urls<const N: usize>(values: [&str; N]) -> Vec<String> {
    values.into_iter().map(str::to_owned).collect()
}
