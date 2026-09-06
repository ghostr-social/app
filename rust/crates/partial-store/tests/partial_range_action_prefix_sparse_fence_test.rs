use crate::partial_range_store::ResponseOpenResult;
use crate::tests::store_fixture;
use ghostr_engine::ByteRange;

#[tokio::test]
async fn a_readable_whole_prefix_fences_later_sparse_responses() {
    let (root, store, identity) = store_fixture::mode_fixture("prefix-sparse-fence").await;
    let generation = store_fixture::source_generation();
    store
        .accept_generation(&identity, generation.clone())
        .await
        .expect("probe generation");
    store.write_range("post", 0, b"newb").await.expect("probe");
    let whole = store_fixture::open_action_response(&store, &identity, 1).await;
    store
        .write_single_response_for_action(&identity, &whole, 0, b"newbyt")
        .await
        .expect("matching whole prefix");
    assert_eq!(
        store
            .read_range("post", 0..6)
            .await
            .expect("read published store state"),
        Some(b"newbyt".to_vec())
    );

    let sparse = store
        .reserve_action(&identity, 2, 2)
        .await
        .expect("sparse reservation");
    assert_eq!(
        store
            .open_sparse_response(&identity, &sparse, generation, ByteRange::new(6, 8))
            .await
            .expect("sparse admission"),
        ResponseOpenResult::RequiresIndependentObject,
        "later sparse bytes have no proof of equivalence to the streaming body"
    );
    whole.revoke();
    store.release_action(&whole).await;
    store.release_action(&sparse).await;
    assert_eq!(store.used_bytes().await, 4);
    assert_eq!(
        store
            .read_range("post", 4..8)
            .await
            .expect("read published store state"),
        None
    );
    store_fixture::discard(&root);
}
