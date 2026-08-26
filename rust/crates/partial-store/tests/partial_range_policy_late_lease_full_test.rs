use sha2::{Digest as _, Sha256};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn playback_lease_acquired_before_whole_policy_discard_preserves_the_object() {
    let mut fixture =
        crate::tests::paused_fixture::paused_store_with_budget("policy-full-late-lease", 11);
    seed_video(&fixture.root).await;
    fixture
        .store
        .load_existing()
        .await
        .expect("valid test fixture");
    let store = std::sync::Arc::clone(&fixture.store);
    let pressure = tokio::spawn(async move { store.enforce_capacity().await });
    fixture.wait_until_admission().await;
    let full = 0..12;
    let mut eviction = Box::pin(
        fixture
            .store
            .evict_ranges("clip", core::slice::from_ref(&full)),
    );
    tokio::select! {
        result = &mut eviction => panic!("policy eviction completed early: {result:?}"),
        () = tokio::task::yield_now() => {}
    }
    let lease = fixture.store.lease("clip");
    fixture.resume();

    assert_eq!(pressure.await.expect("valid test fixture"), 0);
    assert!(eviction.await.is_err());
    assert_eq!(
        fixture
            .store
            .read_range("clip", 0..12)
            .await
            .expect("valid test fixture"),
        Some(b"abcdefghijkl".to_vec())
    );
    drop(lease);
    crate::tests::store_fixture::discard(&fixture.root);
}

async fn seed_video(root: &std::path::Path) {
    let bytes = b"abcdefghijkl";
    tokio::fs::create_dir_all(root)
        .await
        .expect("valid test fixture");
    tokio::fs::write(root.join("clip.part"), bytes)
        .await
        .expect("valid test fixture");
    let digest = format!("{:x}", Sha256::digest(bytes));
    let manifest = format!(
        r#"{{"version":2,"total_len":12,"intervals":[{{"start":0,"end":12,"sha256":"{digest}"}}]}}"#
    );
    tokio::fs::write(root.join("clip.ranges.json"), manifest)
        .await
        .expect("valid test fixture");
}
