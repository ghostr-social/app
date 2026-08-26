use super::{TransformJobs, TransformRequest};
use crate::manager::transfers::InternalEvent;
use crate::transform::{
    TransformBackend, TransformControl, TransformInput, TransformLimits, TransformOutput,
    TransformProfile,
};
use ghostr_engine::adaptive::TransformKind;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{ActionId, DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::{ContentRevision, PartialRangeStore};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

struct PassBackend;

impl TransformBackend for PassBackend {
    fn profile(&self) -> TransformProfile {
        TransformProfile::new(
            TransformKind::Remux,
            TransformLimits::try_new(16, 16, 5, 10).expect("valid test fixture"),
        )
    }

    fn transform(
        &self,
        input: TransformInput<'_>,
        _control: &TransformControl,
    ) -> anyhow::Result<TransformOutput> {
        TransformOutput::try_new(input.bytes().to_vec())
    }
}

#[tokio::test]
async fn runtime_rejects_cross_post_transform_while_one_is_linked() {
    let (events, _receiver) = mpsc::unbounded_channel::<InternalEvent>();
    let resources = super::resource_test_fixture::control();
    let mut jobs = TransformJobs::new(Some(Arc::new(PassBackend)), events, resources);
    let store = Arc::new(PartialRangeStore::with_capacity(
        std::env::temp_dir().join(format!(
            "ghostr-transform-singleflight-{}",
            std::process::id()
        )),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ));

    assert!(jobs.launch(std::sync::Arc::clone(&store), request("first", 1)));
    assert!(!jobs.launch(store, request("second", 2)));
    assert_eq!(jobs.clear(), 1);
}

fn request(id: &str, action: u64) -> TransformRequest {
    let binding = Catalog::new().upsert(
        PostId::new(id),
        VideoMeta {
            urls: vec![format!("https://media.example/{id}.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(1),
            duration_ms: Some(1),
        },
    );
    TransformRequest {
        action: ActionId::new(action),
        binding,
        revision: ContentRevision::default(),
        total: 1,
        kind: TransformKind::Remux,
    }
}
