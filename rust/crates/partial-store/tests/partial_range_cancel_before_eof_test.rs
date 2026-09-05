use crate::tests::store_fixture;
use ghostr_engine::representation::HttpGenerationAuthority;

#[tokio::test]
async fn exact_length_before_eof_does_not_make_a_cancelled_whole_response_complete() {
    let (root, store, identity) = store_fixture::mode_fixture("cancel-before-eof").await;
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
        .write_single_response_for_action(&identity, &action, 0, b"abcdefgh")
        .await
        .expect("fixture");

    action.revoke();
    store.release_action(&action).await;

    assert!(!store.is_complete("post").await.expect("fixture"));
    assert_eq!(store.read_range("post", 0..8).await.expect("fixture"), None);
    assert!(store
        .media_snapshot("post")
        .await
        .expect("fixture")
        .planning_ranges()
        .is_empty());
    store_fixture::discard(&root);
}
