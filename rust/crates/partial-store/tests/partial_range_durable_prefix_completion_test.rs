use crate::tests::store_fixture;
use ghostr_engine::representation::HttpGenerationAuthority;

#[tokio::test]
async fn durable_prefix_eof_preserves_the_player_revision_without_sparse_authority() {
    let (root, store, identity) = store_fixture::mode_fixture("durable-prefix-eof").await;
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
    let playing = store
        .media_snapshot("post")
        .await
        .expect("playing snapshot");
    assert_eq!(
        store.read_range("post", 0..6).await.expect("prefix"),
        Some(b"newbyt".to_vec())
    );
    assert!(!playing.is_complete());
    store
        .write_single_response_for_action(&identity, &action, 6, b"es")
        .await
        .expect("body end");
    assert!(store
        .finish_single_response_for_action(&identity, &action, Some(8), true)
        .await
        .expect("exact EOF"));
    let completed = store
        .media_snapshot("post")
        .await
        .expect("completed snapshot");
    assert!(completed.is_complete());
    assert_eq!(
        completed.revision(),
        playing.revision(),
        "EOF must not invalidate a playing prefix"
    );
    assert_eq!(completed.binding(), playing.binding());
    assert_eq!(
        store.read_range("post", 0..8).await.expect("complete"),
        Some(b"newbytes".to_vec())
    );
    store.release_action(&action).await;
    assert_eq!(
        store.used_bytes().await,
        8,
        "the replaced probe is no longer charged"
    );
    store_fixture::discard(&root);
}
