use crate::partial_range_store::ResponseOpenResult;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::HttpGenerationAuthority;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

const URL: &str = "https://media.example/video.mp4";

#[tokio::test]
async fn admitted_generation_cannot_open_after_http_authority_changes() {
    let root = crate::tests::store_fixture::temp_root("durable-response-lease");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), metadata());
    let identity = binding.transfer(URL).expect("valid test fixture");
    store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    store
        .select_transfer(identity.clone())
        .await
        .expect("valid test fixture");
    let v1 = crate::tests::store_fixture::http_generation(URL, "v1", 1);
    store
        .apply_http_generation(&identity, v1.clone())
        .await
        .expect("valid test fixture");
    let HttpGenerationAuthority::Trusted(admitted) = v1 else {
        unreachable!()
    };
    let action = store
        .reserve_action(&identity, 1, 8)
        .await
        .expect("valid test fixture");
    store
        .apply_http_generation(
            &identity,
            crate::tests::store_fixture::http_generation(URL, "v2", 2),
        )
        .await
        .expect("valid test fixture");

    assert_eq!(
        store
            .open_durable_single_response(
                &identity,
                &action,
                crate::tests::store_fixture::exact_response(8),
                admitted,
            )
            .await
            .expect("valid test fixture"),
        ResponseOpenResult::Stale
    );
    crate::tests::store_fixture::discard(&root);
}

fn metadata() -> VideoMeta {
    VideoMeta {
        urls: vec![URL.into()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
