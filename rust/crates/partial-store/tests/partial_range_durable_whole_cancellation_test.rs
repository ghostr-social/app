use crate::tests::store_fixture;
use ghostr_engine::representation::HttpGenerationAuthority;

#[tokio::test]
async fn cancelled_versioned_whole_response_keeps_a_resumable_prefix() {
    let (root, store, identity) = store_fixture::mode_fixture("durable-whole-cancel").await;
    let authority = store_fixture::http_generation(identity.source().as_str(), "v1", 1);
    store
        .apply_http_generation(&identity, authority.clone())
        .await
        .expect("fixture");
    let HttpGenerationAuthority::Trusted(lease) = authority else {
        unreachable!()
    };
    let action = store
        .reserve_action(&identity, 1, 8)
        .await
        .expect("fixture");
    store
        .open_durable_single_response(&identity, &action, store_fixture::exact_response(8), lease)
        .await
        .expect("fixture");
    store
        .write_single_response_for_action(&identity, &action, 0, b"abcd")
        .await
        .expect("fixture");

    action.revoke();
    store.release_action(&action).await;

    assert_eq!(
        store.read_range("post", 0..4).await.expect("fixture"),
        Some(b"abcd".to_vec())
    );
    assert!(!store.is_complete("post").await.expect("fixture"));
    let continuation = store
        .continuation_for(&identity)
        .await
        .expect("fixture")
        .expect("fixture");
    assert_eq!(continuation.strong_etag(), "\"v1\"");
    assert_eq!(continuation.total_bytes(), 8);
    let snapshot = store.media_snapshot("post").await.expect("fixture");
    assert_eq!(snapshot.planning_ranges(), core::slice::from_ref(&(0..4)));
    assert_eq!(
        snapshot.continuation_source(),
        Some(identity.source().as_str())
    );
    store_fixture::discard(&root);
}
