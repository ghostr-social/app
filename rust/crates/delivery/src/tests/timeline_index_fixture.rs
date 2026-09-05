use crate::chunk::downloader::{
    HttpResponseEvidence, OpenedResponse, ResponseObservation, ResponseWriteMode,
};
use crate::manager::timeline::{TimelineCoordinator, TimelineJobOutcome, TimelineTerminal};
use crate::tests::demand_lease_fixture::{binding, catalog};
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::representation::{
    HttpGenerationAuthority, HttpGenerationKey, HttpGenerationLease, RepresentationBinding,
    SourceGeneration,
};
use ghostr_engine::{ByteRange, PostId};
use ghostr_net::media_retention::MediaRetention;
use ghostr_partial_store::partial_range_store::{capacity::StoreCapacity, PartialRangeStore};
use std::sync::Arc;

pub(super) async fn store(
    root: &std::path::Path,
) -> (Arc<PartialRangeStore>, RepresentationBinding) {
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.to_owned(),
        Arc::default(),
        StoreCapacity::system(u64::MAX),
    ));
    let binding = binding(&catalog(&["post"]), "post");
    store
        .bind_representation(binding.clone())
        .await
        .expect("fixture");
    (store, binding)
}

pub(super) async fn observe(
    store: &PartialRangeStore,
    binding: &RepresentationBinding,
    coordinator: &mut TimelineCoordinator,
    etag: &str,
) {
    let url = "https://post.example/video.mp4";
    let identity = binding.transfer(url).expect("fixture");
    let validator = EvidenceValidator::strong_etag(etag).expect("fixture");
    let key = HttpGenerationKey::try_new(url, Some(validator.clone())).expect("fixture");
    let authority =
        HttpGenerationAuthority::Trusted(HttpGenerationLease::try_new(key, 1).expect("fixture"));
    store
        .apply_http_generation(&identity, authority)
        .await
        .expect("fixture");
    coordinator.observe_index_source(
        &identity,
        &OpenedResponse::new(
            ResponseObservation::Partial {
                range: ByteRange::new(0, 8),
                total: Some(1_000),
            },
            Some(SourceGeneration::try_new(url, etag, 1_000).expect("fixture")),
            ResponseWriteMode::Sparse,
            HttpResponseEvidence {
                final_url: url.into(),
                status: 206,
                content_type: None,
                validator: Some(validator),
                observed: 1.into(),
            },
        ).with_retention(MediaRetention::Public),
    );
    store.set_total_len("post", 1_000).await.expect("fixture");
}

pub(super) async fn run(
    store: &PartialRangeStore,
    binding: &RepresentationBinding,
    coordinator: &mut TimelineCoordinator,
) -> Option<ghostr_engine::media_timeline::MediaTimeline> {
    let snapshot = store.media_snapshot("post").await.expect("fixture");
    let evidence = coordinator.evidence(binding, &snapshot).expect("fixture");
    coordinator.schedule(PostId::new("post"), evidence.clone());
    coordinator.dispatch(&[PostId::new("post")]);
    let result = tokio::time::timeout(core::time::Duration::from_secs(2), coordinator.recv())
        .await
        .expect("fixture")
        .expect("fixture");
    match coordinator.validate(result, Some(&evidence)) {
        Some(TimelineJobOutcome::Terminal(TimelineTerminal::Ready(timeline))) => Some(*timeline),
        _ => None,
    }
}
