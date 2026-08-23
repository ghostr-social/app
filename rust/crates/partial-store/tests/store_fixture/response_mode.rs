use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::{SourceGeneration, TransferIdentity};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

const URL: &str = "https://cdn.example/video";

pub async fn mode_fixture(prefix: &str) -> (PathBuf, PartialRangeStore, TransferIdentity) {
    let root = super::temp_root(prefix);
    let store = super::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), metadata());
    let transfer = binding.transfer(URL).unwrap();
    store.bind_representation(binding).await.unwrap();
    store.select_transfer(transfer.clone()).await.unwrap();
    (root, store, transfer)
}

pub fn source_generation() -> SourceGeneration {
    SourceGeneration::try_new(URL, "\"generation\"", 8).unwrap()
}

fn metadata() -> VideoMeta {
    VideoMeta {
        urls: vec![URL.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
