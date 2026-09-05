use crate::tests::{compiled_index_fixture as index, store_fixture};

#[tokio::test]
async fn source_bound_index_survives_payload_eviction_and_restart_with_shared_accounting() {
    let fixture =
        store_fixture::spaced_store("compiled-index", store_fixture::limits(100_000, 0), 100_000);
    let timeline = index::timeline();
    let key = index::key("\"v1\"");
    fixture
        .store
        .write_range("payload", 0, &[1; 1_000])
        .await
        .expect("fixture");
    fixture
        .store
        .retain_compiled_index(&key, &timeline)
        .await
        .expect("fixture");
    let index_bytes = fixture.store.used_bytes().await - 1_000;
    assert!(index_bytes > 0);
    fixture
        .store
        .evict_ranges("payload", core::slice::from_ref(&(0..1_000)))
        .await
        .expect("fixture");
    assert_eq!(fixture.store.used_bytes().await, index_bytes);

    let reopened = store_fixture::reopened(&fixture);
    reopened.store.load_existing().await.expect("fixture");
    assert_eq!(reopened.store.used_bytes().await, index_bytes);
    assert_eq!(
        reopened.store.compiled_index(&key).await.expect("fixture"),
        Some(timeline)
    );
    assert_eq!(
        reopened
            .store
            .compiled_index(&index::key("\"v2\""))
            .await
            .expect("fixture"),
        None
    );
    reopened.store.clear().await.expect("fixture");
    assert_eq!(
        reopened.store.compiled_index(&key).await.expect("fixture"),
        None
    );
    store_fixture::discard(&fixture.root);
}
