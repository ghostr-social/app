use super::progress::Pending;
use super::{Active, SegmentedDelivery, SegmentedDone};
use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::segmented::fetch::{FetchedObject, OriginTelemetry};
use crate::segmented::{SegmentedCache, SegmentedPhase};
use ghostr_engine::adaptive::{DecisionOutcome, ResourceCost};
use ghostr_engine::origin_model::{NetworkClass, OriginOutcome};
use ghostr_engine::{ActionId, DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use std::time::Duration;

const MANIFEST: &str =
    "#EXTM3U\n#EXT-X-TARGETDURATION:4\n#EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n";

#[tokio::test]
async fn queued_success_is_superseded_without_losing_origin_or_resource_truth() {
    let cache = SegmentedCache::new();
    let mut delivery = SegmentedDelivery::new(cache.clone());
    delivery.apply_focus(&focus(1, "https://old.example/root.m3u8"));
    let post = PostId::new("stream");
    delivery.active.insert(post.clone(), completed_active());

    delivery.apply_focus(&focus(2, "https://new.example/root.m3u8"));
    assert!(!delivery.active[&post].cancelling);
    let finish = delivery.finish(succeeded(post.clone())).unwrap();

    assert_eq!(finish.outcome, DecisionOutcome::Superseded);
    assert_eq!(
        finish.actual_resources,
        Some(ResourceCost::new(MANIFEST.len() as u64, 0, 0, 1))
    );
    assert_eq!(finish.observation.unwrap().outcome, OriginOutcome::Success);
    assert_eq!(cache.snapshot("stream").phase, SegmentedPhase::Queued);
    assert_eq!(delivery.pending[&post].generation, 2);
}

pub(super) fn completed_active() -> Active {
    let (cancellation, cancelled) = tokio::sync::oneshot::channel();
    drop(cancelled);
    Active {
        action: ActionId::new(7),
        pending: Pending::root(1, 1, 0, "https://old.example/root.m3u8".to_owned()),
        committed_until_ms: u64::MAX,
        _task: tokio::spawn(std::future::pending()),
        cancellation: Some(cancellation),
        cancelling: false,
    }
}

pub(super) fn succeeded(post: PostId) -> SegmentedDone {
    SegmentedDone {
        action: ActionId::new(7),
        post,
        generation: 1,
        outcome: Ok(FetchedObject {
            request_url: "https://old.example/root.m3u8".to_owned(),
            final_url: "https://old.example/root.m3u8".parse().unwrap(),
            body: Arc::from(MANIFEST.as_bytes()),
            content_type: Some("application/vnd.apple.mpegurl".to_owned()),
            cache: Default::default(),
            telemetry: telemetry(),
            offset: 0,
            continuation: None,
        }),
        observed_at_ms: 10,
        resources: Default::default(),
    }
}

fn telemetry() -> OriginTelemetry {
    OriginTelemetry {
        elapsed: Duration::from_millis(25),
        ttfb: Some(Duration::from_millis(10)),
        concurrency: 1,
        network_class: NetworkClass::Wifi,
    }
}

pub(super) fn focus(generation: u64, source: &str) -> DeliveryFocus {
    DeliveryFocus {
        items: vec![FocusItem {
            post: PostId::new("stream"),
            meta: VideoMeta {
                urls: vec![source.to_owned()],
                delivery: DeliveryKind::Hls,
                sha256: None,
                size_bytes: None,
                duration_ms: Some(4_000),
            },
        }],
        previews: Vec::new(),
        current_index: 0,
        watch_ms: 0,
        generation: FocusGeneration::try_new(generation).unwrap(),
        transition: FocusTransition::RosterChange,
        rescue: None,
    }
}
