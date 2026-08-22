use super::TransformRequest;
use ghostr_engine::adaptive::TransformKind;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{ActionId, DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::{
    ContentRevision, PartialRangeStore, TransformFence, TransformPublication,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

pub(super) struct TransformFixture {
    pub(super) store: Arc<PartialRangeStore>,
    root: PathBuf,
    binding: RepresentationBinding,
    revision: ContentRevision,
}

impl TransformFixture {
    pub(super) async fn seeded(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("{label}-{nonce}"));
        std::fs::create_dir_all(&root).unwrap();
        let store = Arc::new(PartialRangeStore::with_capacity(
            root.clone(),
            Arc::new(Mutex::new(0)),
            StoreCapacity::system(u64::MAX),
        ));
        let binding = binding();
        store.bind_representation(binding.clone()).await.unwrap();
        store.set_total_len("post", 4).await.unwrap();
        store.write_range("post", 0, b"data").await.unwrap();
        store.finalize("post", None).await.unwrap();
        let revision = store.media_snapshot("post").await.unwrap().revision();
        Self {
            store,
            root,
            binding,
            revision,
        }
    }

    pub(super) fn request(&self, action: u64) -> TransformRequest {
        TransformRequest {
            action: ActionId::new(action),
            binding: self.binding.clone(),
            revision: self.revision,
            total: 4,
            kind: TransformKind::Remux,
        }
    }

    pub(super) fn publication(&self) -> TransformPublication {
        TransformPublication::try_new(
            TransformFence::new(self.binding.clone(), self.revision),
            TransformKind::Remux,
            b"done!".to_vec(),
            16,
        )
        .unwrap()
    }

    pub(super) fn has_transform_staging(&self) -> bool {
        [
            "post.transform.video",
            "post.transform.ranges",
            "post.transform.representation",
            "post.transform.record",
        ]
        .iter()
        .any(|name| self.root.join(name).exists())
    }
}

impl Drop for TransformFixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).ok();
    }
}

fn binding() -> RepresentationBinding {
    Catalog::new().upsert(
        PostId::new("post"),
        VideoMeta {
            urls: vec!["https://media.example/post.mp4".to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(4),
            duration_ms: Some(1),
        },
    )
}
