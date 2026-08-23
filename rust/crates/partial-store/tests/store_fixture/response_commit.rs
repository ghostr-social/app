use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

const URL: &str = "https://media.example/video.mp4";

pub async fn staged_replacement(
    prefix: &str,
) -> (PathBuf, PartialRangeStore, RepresentationBinding) {
    let root = super::temp_root(prefix);
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), metadata());
    let identity = binding.transfer(URL).unwrap();
    let store = super::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    prepare_canonical(&store, &binding, &identity).await;
    prepare_session(&store, &identity).await;
    (root, store, binding)
}

async fn prepare_canonical(
    store: &PartialRangeStore,
    binding: &RepresentationBinding,
    identity: &ghostr_engine::representation::TransferIdentity,
) {
    store.bind_representation(binding.clone()).await.unwrap();
    store.select_transfer(identity.clone()).await.unwrap();
    store
        .apply_http_generation(&identity, super::http_generation(URL, "v1", 1))
        .await
        .unwrap();
    super::publish_whole(&store, &identity, 1, b"oldbytes").await;
    store.finalize("post", None).await.unwrap();
}

async fn prepare_session(
    store: &PartialRangeStore,
    identity: &ghostr_engine::representation::TransferIdentity,
) {
    let action = store.reserve_action(&identity, 2, 8).await.unwrap();
    store
        .open_action_scoped_single_response(&identity, &action, super::exact_response(8))
        .await
        .unwrap();
    store
        .write_single_response_for_action(&identity, &action, 0, b"newbytes")
        .await
        .unwrap();
    store
        .finish_single_response_for_action(&identity, &action, Some(8), true)
        .await
        .unwrap();
}

pub async fn backup_canonical(root: &Path) {
    tokio::fs::rename(root.join("post.video"), root.join("post.video.prev"))
        .await
        .unwrap();
    tokio::fs::rename(root.join("post.ranges.json"), root.join("post.ranges.prev"))
        .await
        .unwrap();
    tokio::fs::rename(
        root.join("post.http-generation.json"),
        root.join("post.http-generation.prev"),
    )
    .await
    .unwrap();
}

pub fn response_commit(phase: &str) -> String {
    let digest = format!("{:x}", Sha256::digest(b"newbytes"));
    format!(
        "{{\"version\":1,\"phase\":\"{phase}\",\"target\":\"verified\",\"total\":8,\"sha256\":\"{digest}\",\"retire_http\":true}}"
    )
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
