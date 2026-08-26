use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{ByteRange, DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn staged_whole_publish_retires_older_sparse_action_without_deleting_the_result() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "staged-sparse-takeover",
        crate::tests::store_fixture::limits(16, 0),
        16,
    );
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let sparse_identity = binding
        .transfer("https://a.example/clip")
        .expect("valid test fixture");
    let whole_identity = binding
        .transfer("https://b.example/clip")
        .expect("valid test fixture");
    let generation = SourceGeneration::try_new(sparse_identity.source().as_str(), "\"a\"", 8)
        .expect("valid test fixture");
    fixture
        .store
        .bind_representation(binding.clone())
        .await
        .expect("valid test fixture");

    let sparse = fixture
        .store
        .reserve_action(&sparse_identity, 1, 8)
        .await
        .expect("valid test fixture");
    fixture
        .store
        .open_sparse_response(
            &sparse_identity,
            &sparse,
            generation.clone(),
            ByteRange::new(0, 8),
        )
        .await
        .expect("valid test fixture");
    fixture
        .store
        .write_range_for_action_if_current(&sparse_identity, &generation, &sparse, 0, b"old!")
        .await
        .expect("valid test fixture");
    let whole = fixture
        .store
        .reserve_action(&whole_identity, 2, 8)
        .await
        .expect("valid test fixture");
    fixture
        .store
        .open_single_response_for_action(
            &whole_identity,
            &whole,
            crate::tests::store_fixture::exact_response(8),
        )
        .await
        .expect("valid test fixture");
    fixture
        .store
        .write_single_response_for_action(&whole_identity, &whole, 0, b"new data")
        .await
        .expect("valid test fixture");
    assert!(fixture
        .store
        .finish_single_response_for_action(&whole_identity, &whole, Some(8), true)
        .await
        .expect("valid test fixture"));
    fixture.store.release_action(&whole).await;

    assert!(!fixture
        .store
        .write_range_for_action_if_current(&sparse_identity, &generation, &sparse, 4, b"late")
        .await
        .expect("valid test fixture"));
    fixture.store.release_action(&sparse).await;
    assert_eq!(fixture.store.used_bytes().await, 8);
    assert!(!fixture.root.join("clip.sparse.intent").exists());
    assert_eq!(
        fixture
            .store
            .read_range("clip", 0..8)
            .await
            .expect("valid test fixture"),
        Some(b"new data".to_vec())
    );

    let reopened = crate::tests::store_fixture::reopened(&fixture);
    reopened
        .store
        .load_existing()
        .await
        .expect("valid test fixture");
    reopened
        .store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    assert_eq!(
        reopened
            .store
            .read_range("clip", 0..8)
            .await
            .expect("valid test fixture"),
        Some(b"new data".to_vec())
    );
    crate::tests::store_fixture::discard(&fixture.root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec![
            "https://a.example/clip".to_owned(),
            "https://b.example/clip".to_owned(),
        ],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
