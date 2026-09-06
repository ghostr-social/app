use crate::tests::store_fixture;
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::representation::{
    HttpGenerationAuthority, HttpGenerationKey, HttpGenerationLease, RequestSelection,
};

#[tokio::test]
async fn cancelled_whole_preserves_selection_before_sparse_continuation() {
    let (root, store, identity) = store_fixture::mode_fixture("selected-whole-cancel").await;
    let selection = Some(RequestSelection::new([7; 32]));
    let key = HttpGenerationKey::try_new(
        identity.source().as_str(),
        EvidenceValidator::strong_etag("\"v1\""),
    )
    .expect("key")
    .with_request_selection(selection);
    let lease = HttpGenerationLease::try_new(key, 1).expect("lease");
    store
        .apply_http_generation(&identity, HttpGenerationAuthority::Trusted(lease.clone()))
        .await
        .expect("generation");
    let action = store.reserve_action(&identity, 1, 8).await.expect("action");
    store
        .open_durable_single_response(&identity, &action, store_fixture::exact_response(8), lease)
        .await
        .expect("open");
    store
        .write_single_response_for_action(&identity, &action, 0, b"abcd")
        .await
        .expect("write");
    action.revoke();
    store.release_action(&action).await;
    let continuation = store
        .continuation_for(&identity)
        .await
        .expect("lookup")
        .expect("prefix");
    assert_eq!(continuation.request_selection(), selection);
    assert!(
        store
            .http_generation_matches_source(&identity, &continuation)
            .await
    );
    let changed = continuation.with_request_selection(Some(RequestSelection::new([8; 32])));
    assert!(
        !store
            .http_generation_matches_source(&identity, &changed)
            .await
    );
    store_fixture::discard(&root);
}
