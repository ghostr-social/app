mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::HttpGenerationAuthority;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::ResponseOpenResult;
use std::sync::Arc;
use tokio::sync::Mutex;

const URL: &str = "https://media.example/video.mp4";

#[tokio::test]
async fn admitted_generation_cannot_open_after_http_authority_changes() {
    let root = store_fixture::temp_root("durable-response-lease");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), metadata());
    let identity = binding.transfer(URL).unwrap();
    store.bind_representation(binding).await.unwrap();
    store.select_transfer(identity.clone()).await.unwrap();
    let v1 = store_fixture::http_generation(URL, "v1", 1);
    store
        .apply_http_generation(&identity, v1.clone())
        .await
        .unwrap();
    let HttpGenerationAuthority::Trusted(admitted) = v1 else {
        unreachable!()
    };
    let action = store.reserve_action(&identity, 1, 8).await.unwrap();
    store
        .apply_http_generation(&identity, store_fixture::http_generation(URL, "v2", 2))
        .await
        .unwrap();

    assert_eq!(
        store
            .open_durable_single_response(
                &identity,
                &action,
                store_fixture::exact_response(8),
                admitted,
            )
            .await
            .unwrap(),
        ResponseOpenResult::Stale
    );
    store_fixture::discard(&root);
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
