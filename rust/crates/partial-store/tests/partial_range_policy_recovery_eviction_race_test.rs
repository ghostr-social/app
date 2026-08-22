#![cfg(unix)]

#[path = "store_fixture/paused.rs"]
mod paused_fixture;
mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use sha2::{Digest, Sha256};
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn policy_recovery_cannot_reinstall_an_object_evicted_in_parallel() {
    let mut fixture = paused_fixture::paused_store_with_budget("policy-recovery-race", 19);
    let (binding, identity) = identity();
    fixture.store.bind_representation(binding).await.unwrap();
    seed_debt(&fixture.root).await;
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o500)).unwrap();
    fixture.store.load_existing().await.unwrap();
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o700)).unwrap();
    let store = fixture.store.clone();
    let pressure = tokio::spawn(async move { store.enforce_capacity().await });
    fixture.wait_until_admission().await;
    let store = fixture.store.clone();
    let recovery = tokio::spawn(async move { store.reserve_action(&identity, 1, 1).await });
    wait_for_recovery(&fixture.root).await;
    fixture.resume();

    assert_eq!(pressure.await.unwrap(), 0);
    let action = recovery.await.unwrap().unwrap();
    assert_eq!(
        fixture.store.read_range("clip", 0..12).await.unwrap(),
        Some(b"abcdefghijkl".to_vec())
    );
    assert_eq!(fixture.store.used_bytes().await, 12);
    fixture.store.release_action(&action).await;
    store_fixture::discard(&fixture.root);
}

async fn wait_for_recovery(root: &std::path::Path) {
    let deadline = tokio::time::Instant::now() + Duration::from_millis(100);
    while root.join("clip.part.evict").exists() && tokio::time::Instant::now() < deadline {
        tokio::task::yield_now().await;
    }
}

async fn seed_debt(root: &std::path::Path) {
    let bytes = b"abcdefghijkl";
    tokio::fs::write(root.join("clip.part"), bytes)
        .await
        .unwrap();
    let digest = format!("{:x}", Sha256::digest(bytes));
    let manifest = format!(
        r#"{{"version":2,"total_len":12,"intervals":[{{"start":0,"end":12,"sha256":"{digest}"}}]}}"#
    );
    tokio::fs::write(root.join("clip.ranges.json"), manifest)
        .await
        .unwrap();
    tokio::fs::write(root.join("clip.part.evict"), b"abcdefgh")
        .await
        .unwrap();
    tokio::fs::write(
        root.join("clip.evict.intent"),
        br#"{"version":1,"retained_bytes":8}"#,
    )
    .await
    .unwrap();
}

fn identity() -> (
    ghostr_engine::representation::RepresentationBinding,
    ghostr_engine::representation::TransferIdentity,
) {
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let identity = binding.transfer("https://cdn.example/video").unwrap();
    (binding, identity)
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://cdn.example/video".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(12),
        duration_ms: Some(1_000),
    }
}
