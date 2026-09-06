use crate::partial_range_store::{PartialRangeStore, TransformFence, TransformPublication};
use crate::tests::store_fixture;
use ghostr_engine::adaptive::TransformKind;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

pub(super) struct TransformFixture {
    pub(super) root: PathBuf,
    pub(super) store: Arc<PartialRangeStore>,
    binding: RepresentationBinding,
}

impl TransformFixture {
    pub(super) async fn new() -> Self {
        let root = store_fixture::temp_root("transform-owner-recovery");
        let store = Arc::new(store_fixture::plain_store(
            root.clone(),
            Arc::new(Mutex::new(0)),
        ));
        let binding = input_binding();
        store
            .bind_representation(binding.clone())
            .await
            .expect("input binding");
        store.set_total_len("post", 5).await.expect("input length");
        store
            .write_range("post", 0, b"input")
            .await
            .expect("input bytes");
        store.finalize("post", None).await.expect("completed input");
        Self {
            root,
            store,
            binding,
        }
    }

    pub(super) async fn publication(&self) -> TransformPublication {
        let revision = self
            .store
            .media_snapshot("post")
            .await
            .expect("input snapshot")
            .revision();
        TransformPublication::try_new(
            TransformFence::new(self.binding.clone(), revision),
            TransformKind::Remux,
            b"output".to_vec(),
            16,
        )
        .expect("bounded output")
    }
}

pub(super) fn files(root: &Path) -> Vec<OsString> {
    let mut files: Vec<_> = std::fs::read_dir(root)
        .expect("store directory")
        .map(|entry| entry.expect("store artifact").file_name())
        .collect();
    files.sort();
    files
}

fn input_binding() -> RepresentationBinding {
    Catalog::new().upsert(
        PostId::new("post"),
        VideoMeta {
            urls: vec!["https://origin.example/input.mp4".to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(5),
            duration_ms: Some(1_000),
        },
    )
}
