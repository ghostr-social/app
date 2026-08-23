use super::super::{transaction, TransformFence, TransformPublication};
use super::test_paths::temp_root;
use crate::partial_range_disk as disk;
use crate::partial_range_paths::StorePaths;
use crate::partial_range_store::capacity::StoreCapacity;
use crate::partial_range_store::PartialRangeStore;
use ghostr_engine::adaptive::TransformKind;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::representation::{
    HttpGenerationAuthority, HttpGenerationKey, HttpGenerationLease, RepresentationBinding,
};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

const URL: &str = "https://origin.example/input.mp4";

pub(super) async fn interrupted_transaction() -> (PathBuf, RepresentationBinding) {
    let root = temp_root();
    let input = input_binding();
    let store = store(root.clone());
    store.bind_representation(input.clone()).await.unwrap();
    authorize_input(&store, &input).await;
    store.write_range("post", 0, b"input").await.unwrap();
    store.set_total_len("post", 5).await.unwrap();
    store.finalize("post", None).await.unwrap();
    let revision = store.media_snapshot("post").await.unwrap().revision();
    let publication = TransformPublication::try_new(
        TransformFence::new(input.clone(), revision),
        TransformKind::Remux,
        b"output".to_vec(),
        16,
    )
    .unwrap();
    transaction::stage(&store.paths, "post", publication)
        .await
        .unwrap();
    interrupt_commit(&store.paths).await;
    (root, input)
}

async fn authorize_input(store: &PartialRangeStore, binding: &RepresentationBinding) {
    let identity = binding.transfer(URL).unwrap();
    let validator = EvidenceValidator::strong_etag("\"generation-1\"");
    let key = HttpGenerationKey::try_new(URL, validator).unwrap();
    let lease = HttpGenerationLease::try_new(key, 1).unwrap();
    let authority = HttpGenerationAuthority::Trusted(lease);
    assert!(store
        .apply_http_generation(&identity, authority)
        .await
        .unwrap());
}

pub(super) async fn assert_rolled_back(root: &Path, input: &RepresentationBinding) {
    let reopened = store(root.to_owned());
    reopened.load_existing().await.unwrap();
    reopened.bind_representation(input.clone()).await.unwrap();
    assert_eq!(
        reopened.media_snapshot("post").await.unwrap().binding(),
        Some(input)
    );
    assert_eq!(
        reopened.read_range("post", 0..5).await.unwrap(),
        Some(b"input".to_vec())
    );
    assert!(disk::file_len(&reopened.paths.transform("post").commit())
        .await
        .unwrap()
        .is_none());
}

async fn interrupt_commit(paths: &StorePaths) {
    let transform = paths.transform("post");
    disk::write_marker(&transform.commit()).await.unwrap();
    tokio::fs::rename(paths.completed("post"), transform.data_backup())
        .await
        .unwrap();
    tokio::fs::rename(paths.manifest("post"), transform.manifest_backup())
        .await
        .unwrap();
    tokio::fs::rename(paths.representation("post"), transform.identity_backup())
        .await
        .unwrap();
    tokio::fs::rename(transform.data(), paths.completed("post"))
        .await
        .unwrap();
}

fn store(root: PathBuf) -> PartialRangeStore {
    PartialRangeStore::with_capacity(
        root,
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    )
}

fn input_binding() -> RepresentationBinding {
    Catalog::new().upsert(
        PostId::new("post"),
        VideoMeta {
            urls: vec![URL.into()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(5),
            duration_ms: Some(1_000),
        },
    )
}
