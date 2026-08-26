use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn refused_replacement_preserves_the_previous_source_generation() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "single-response-refusal",
        crate::tests::store_fixture::limits(8, 0),
        1_000,
    );
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let transfer = binding
        .transfer("https://cdn.example/video")
        .expect("valid test fixture");
    fixture
        .store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    fixture
        .store
        .select_transfer(transfer.clone())
        .await
        .expect("valid test fixture");
    fixture
        .store
        .write_range("post", 0, b"old!")
        .await
        .expect("valid test fixture");
    fixture
        .store
        .begin_single_response(&transfer, 9, crate::tests::store_fixture::exact_response(8))
        .await
        .expect("valid test fixture");

    fixture
        .store
        .write_single_response_if_current(&transfer, 9, 0, b"newbytes")
        .await
        .expect_err("staging exceeds the hard store budget");

    assert_eq!(
        fixture
            .store
            .read_range("post", 0..4)
            .await
            .expect("valid test fixture"),
        Some(b"old!".to_vec())
    );
    crate::tests::store_fixture::discard(&fixture.root);
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
