#![cfg(unix)]

mod store_fixture;

use std::os::unix::fs::PermissionsExt;
use store_fixture::{discard, limits, spaced_store};

#[tokio::test]
async fn failed_partial_range_eviction_reports_no_freed_bytes() {
    let fixture = spaced_store("ghostr-eviction-failure", limits(1_000, 400), 1_000);
    fixture
        .store
        .write_range("clip", 0, &[1; 400])
        .await
        .expect("seed clip");
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o500))
        .expect("block eviction");
    fixture.space.set(0);

    let freed = fixture.store.enforce_capacity().await;

    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o700))
        .expect("restore permissions");
    assert_eq!(freed, 0);
    assert_eq!(*fixture.used_bytes.lock().await, 400);
    assert!(fixture.root.join("clip.part").exists());
    assert_eq!(fixture.store.enforce_capacity().await, 400);
    discard(&fixture.root);
}
