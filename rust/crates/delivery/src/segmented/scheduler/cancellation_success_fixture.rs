use super::super::prepared::prepare_transfer;
use super::{test_fence, SegmentedDone};
use crate::segmented::cache::{StageAdmission, StageReservation};
use crate::segmented::fetch::{FetchedObject, OriginTelemetry};
use crate::segmented::SegmentedCache;
use ghostr_engine::adaptive::HlsBootstrapStage;
use ghostr_engine::origin_model::NetworkClass;
use ghostr_engine::{ActionId, PostId};
use std::sync::Arc;
use std::time::Duration;

pub(super) const MANIFEST: &str =
    "#EXTM3U\n#EXT-X-TARGETDURATION:4\n#EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n";
const SOURCE: &str = "https://old.example/root.m3u8";

pub(super) async fn succeeded(cache: &SegmentedCache, post: PostId) -> SegmentedDone {
    let maximum = HlsBootstrapStage::RootManifest.maximum_bytes();
    let fence = test_fence(1, 1, SOURCE, maximum);
    let admission = StageAdmission::new(
        post.clone(),
        fence.clone(),
        500,
        StageReservation::block(maximum),
    );
    let lease = cache.admit_stage(admission).unwrap();
    let (cancel, cancelled) = tokio::sync::oneshot::channel();
    let outcome = prepare_transfer(lease, fetched(), cancelled).await;
    drop(cancel);
    SegmentedDone {
        action: ActionId::new(7),
        post,
        fence,
        outcome,
        observed_at_ms: 10,
        resources: Default::default(),
    }
}

fn fetched() -> FetchedObject {
    FetchedObject {
        request_url: SOURCE.to_owned(),
        final_url: SOURCE.parse().unwrap(),
        body: Arc::from(MANIFEST.as_bytes()),
        content_type: Some("application/vnd.apple.mpegurl".to_owned()),
        cache: Default::default(),
        telemetry: OriginTelemetry {
            elapsed: Duration::from_millis(25),
            ttfb: Some(Duration::from_millis(10)),
            concurrency: 1,
            network_class: NetworkClass::Wifi,
        },
        offset: 0,
        continuation: None,
    }
}
