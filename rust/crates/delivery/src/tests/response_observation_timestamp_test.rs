use crate::chunk::downloader::{
    HttpResponseEvidence, OpenedResponse, ResponseObservation, ResponseWriteMode,
};
use crate::manager::inflight::ChunkAttempt;
use crate::manager::transfers::ObservedResponse;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{ActionId, ByteRange, ChunkId, DeliveryKind, PostId, VideoMeta};

const SOURCE: &str = "https://media.example/video.mp4";

#[test]
fn response_event_keeps_the_network_boundary_timestamp() {
    let observed = ObservedResponse::at_network_boundary(attempt(), response());

    assert_eq!(observed.response.evidence().observed.observed_at_ms, 424_242);
    assert_eq!(observed.response.evidence().observed.order, 7);
}

fn attempt() -> ChunkAttempt {
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), metadata());
    let identity = binding.transfer(SOURCE).expect("source identity");
    ChunkAttempt::new(
        ChunkId {
            post,
            range: ByteRange::new(4, 8),
        },
        identity,
        ActionId::new(1),
    )
}

fn response() -> OpenedResponse {
    OpenedResponse::new(
        ResponseObservation::Ignored {
            total: None,
            range_support: Some(false),
        },
        None,
        ResponseWriteMode::Sparse,
        HttpResponseEvidence {
            final_url: SOURCE.to_owned(),
            status: 200,
            content_type: Some("video/mp4".to_owned()),
            validator: None,
            observed: ghostr_engine::evidence::EvidenceTime::ordered(424_242, 7),
        },
    )
}

fn metadata() -> VideoMeta {
    VideoMeta {
        urls: vec![SOURCE.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: None,
        duration_ms: None,
    }
}
