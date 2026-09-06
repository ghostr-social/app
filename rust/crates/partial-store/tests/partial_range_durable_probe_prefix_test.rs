use crate::tests::store_fixture;
use ghostr_engine::representation::HttpGenerationAuthority;

#[tokio::test]
async fn a_durable_whole_response_streams_past_a_matching_probe_before_eof() {
    let (root, store, identity) = store_fixture::mode_fixture("durable-probe-prefix").await;
    let authority = store_fixture::http_generation(identity.source().as_str(), "generation", 1);
    store
        .apply_http_generation(&identity, authority.clone())
        .await
        .expect("HTTP authority");
    let HttpGenerationAuthority::Trusted(lease) = authority else {
        unreachable!()
    };
    store
        .accept_generation(&identity, store_fixture::source_generation())
        .await
        .expect("probe generation");
    store.write_range("post", 0, b"newb").await.expect("probe");
    let before = store.media_snapshot("post").await.expect("probe snapshot");
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
        store
            .read_range("post", 0..6)
            .await
            .expect("stream before EOF"),
        Some(b"newbyt".to_vec()),
        "a trusted whole response exposes its matching prefix before EOF"
    );
    let after = store.media_snapshot("post").await.expect("stream snapshot");
    assert_eq!(
        after.revision(),
        before.revision(),
        "matching probe keeps playback authority"
    );
    assert!(!after.is_complete(), "the response is still incomplete");
    assert_eq!(
        store.used_bytes().await,
        10,
        "probe and body are both charged"
    );
    action.revoke();
    store.release_action(&action).await;
    assert_eq!(
        store.used_bytes().await,
        4,
        "only the canonical probe survives"
    );
    assert_eq!(
        store
            .read_range("post", 0..4)
            .await
            .expect("probe restored"),
        Some(b"newb".to_vec())
    );
    assert_eq!(
        store
            .read_range("post", 4..6)
            .await
            .expect("suffix revoked"),
        None
    );
    store_fixture::discard(&root);
}
