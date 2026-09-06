use crate::tests::store_fixture;
use ghostr_engine::representation::HttpGenerationAuthority;

#[tokio::test]
async fn renewing_http_authority_withdraws_the_previous_response_prefix() {
    let (root, store, identity) = store_fixture::mode_fixture("durable-prefix-lease").await;
    let authority = store_fixture::http_generation(identity.source().as_str(), "generation", 1);
    store
        .apply_http_generation(&identity, authority.clone())
        .await
        .expect("HTTP authority");
    let HttpGenerationAuthority::Trusted(lease) = authority else {
        unreachable!()
    };
    store.set_total_len("post", 8).await.expect("known length");
    store.write_range("post", 0, b"newb").await.expect("probe");
    let action = store
        .reserve_action(&identity, 1, 8)
        .await
        .expect("reserve");
    store
        .open_durable_single_response(&identity, &action, store_fixture::exact_response(8), lease)
        .await
        .expect("whole response");
    store
        .write_single_response_for_action(&identity, &action, 0, b"newbyt")
        .await
        .expect("partial body");
    assert_eq!(
        store.read_range("post", 0..6).await.expect("prefix"),
        Some(b"newbyt".to_vec())
    );

    let replacement = store_fixture::http_generation(identity.source().as_str(), "generation", 2);
    store
        .apply_http_generation(&identity, replacement)
        .await
        .expect("renew authority");
    assert_eq!(
        store.read_range("post", 0..6).await.expect("stale prefix"),
        None,
        "bytes owned by a superseded response lease are no longer servable"
    );
    assert_eq!(
        store
            .read_range("post", 0..4)
            .await
            .expect("canonical probe"),
        Some(b"newb".to_vec())
    );
    assert!(!store
        .write_single_response_for_action(&identity, &action, 6, b"es")
        .await
        .expect("stale write"));
    action.revoke();
    store.release_action(&action).await;
    assert_eq!(store.used_bytes().await, 4);
    store_fixture::discard(&root);
}
