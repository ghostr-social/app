mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn refused_replacement_preserves_the_previous_source_generation() {
    let fixture = store_fixture::spaced_store(
        "single-response-refusal",
        store_fixture::limits(8, 0),
        1_000,
    );
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let transfer = binding.transfer("https://cdn.example/video").unwrap();
    fixture.store.bind_representation(binding).await.unwrap();
    fixture
        .store
        .select_transfer(transfer.clone())
        .await
        .unwrap();
    fixture.store.write_range("post", 0, b"old!").await.unwrap();
    fixture
        .store
        .begin_single_response(&transfer, 9, store_fixture::exact_response(8))
        .await
        .unwrap();

    fixture
        .store
        .write_single_response_if_current(&transfer, 9, 0, b"newbytes")
        .await
        .expect_err("staging exceeds the hard store budget");

    assert_eq!(
        fixture.store.read_range("post", 0..4).await.unwrap(),
        Some(b"old!".to_vec())
    );
    store_fixture::discard(&fixture.root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://cdn.example/video".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
