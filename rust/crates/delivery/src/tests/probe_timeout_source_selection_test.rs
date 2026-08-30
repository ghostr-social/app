use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::{MetadataProbePool, ProbeClaimQuery};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

const MIRROR_A: &str = "https://a.example/video.mp4";
const MIRROR_B: &str = "https://b.example/video.mp4";

#[test]
fn planner_selected_mirror_can_probe_after_another_mirror_times_out() {
    let post = PostId::new("selection");
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), metadata());
    let retry = RetryBook::new(RetryPolicy::default());
    let mut probes = MetadataProbePool::new(1);
    let timed_out = catalog
        .transfer_identity(&post, MIRROR_A)
        .expect("timed-out mirror identity");
    probes.require_body(&timed_out);

    let claimed = probes.claim_selected(ProbeClaimQuery {
        catalog: &catalog,
        retry: &retry,
        post: &post,
        source: MIRROR_B,
        observed_at_ms: 1,
    });

    assert!(claimed.is_ok(), "planned mirror was refused: {claimed:?}");
}

fn metadata() -> VideoMeta {
    VideoMeta {
        urls: vec![MIRROR_A.into(), MIRROR_B.into()],
        delivery: DeliveryKind::Progressive,
        sha256: Some("digest".into()),
        size_bytes: None,
        duration_ms: None,
    }
}
