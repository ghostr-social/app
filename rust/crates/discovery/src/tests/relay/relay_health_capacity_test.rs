use crate::relay::health::RelayHealth;

const TARGET: &str = "wss://000-target.example";

#[tokio::test(start_paused = true)]
async fn capacity_churn_cannot_evict_an_in_flight_admission() {
    let health = RelayHealth::new();
    let target = health.admit(&urls([TARGET]));
    fill_with_closed_circuits(&health);

    let newcomer = urls(["wss://zzz-new.example"]);
    assert_eq!(health.admit(&newcomer).len(), 1);
    health.observe(&target, &[], &[TARGET.to_owned()]);

    assert!(health.admit(&urls([TARGET])).is_empty());
}

#[tokio::test(start_paused = true)]
async fn capacity_prefers_eviction_of_closed_over_cooling_circuits() {
    let health = RelayHealth::new();
    let dead = health.admit(&urls([TARGET]));
    health.observe(&dead, &[], &[TARGET.to_owned()]);
    fill_with_closed_circuits(&health);

    assert_eq!(health.admit(&urls(["wss://zzz-new.example"])).len(), 1);

    assert!(health.admit(&urls([TARGET])).is_empty());
}

#[tokio::test(start_paused = true)]
async fn all_cooling_capacity_fails_closed_without_bypassing_quarantine() {
    let health = RelayHealth::new();
    for index in 0..256 {
        let url = format!("wss://dead-{index:03}.example");
        let admission = health.admit(std::slice::from_ref(&url));
        health.observe(&admission, &[], &[url]);
    }

    assert!(health
        .admit(&urls(["wss://untracked-new.example"]))
        .is_empty());
    assert!(health.admit(&urls(["wss://dead-000.example"])).is_empty());

    tokio::time::advance(std::time::Duration::from_secs(2)).await;
    assert_eq!(
        health.admit(&urls(["wss://untracked-new.example"])).len(),
        1,
        "expired cooldowns must not permanently deny new routes"
    );
}

#[tokio::test(start_paused = true)]
async fn eviction_cannot_reset_a_later_candidate_recovery_history() {
    let health = RelayHealth::new();
    for index in 0..256 {
        let url = format!("wss://dead-{index:03}.example");
        let admission = health.admit(std::slice::from_ref(&url));
        health.observe(&admission, &[], &[url]);
    }
    tokio::time::advance(std::time::Duration::from_secs(2)).await;
    let candidates = urls(["wss://000-new.example", "wss://dead-000.example"]);

    let admissions = health.admit(&candidates);
    health.observe(&admissions, &[], &candidates);
    tokio::time::advance(std::time::Duration::from_secs(2)).await;

    assert!(health.admit(&urls(["wss://dead-000.example"])).is_empty());
}

fn fill_with_closed_circuits(health: &RelayHealth) {
    for index in 0..255 {
        let url = format!("wss://healthy-{index:03}.example");
        let admission = health.admit(std::slice::from_ref(&url));
        health.observe(&admission, &[url], &[]);
    }
}

fn urls<const N: usize>(values: [&str; N]) -> Vec<String> {
    values.into_iter().map(str::to_owned).collect()
}
