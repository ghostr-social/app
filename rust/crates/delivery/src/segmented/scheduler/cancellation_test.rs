use super::progress::Pending;
use super::{active_network, old_root_fence, Active, SegmentedDelivery, SegmentedDone};
use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::segmented::fetch::{FetchFailure, OriginTelemetry};
use crate::segmented::SegmentedCache;
use ghostr_engine::adaptive::{DecisionOutcome, ResourceCost};
use ghostr_engine::origin_model::NetworkClass;
use ghostr_engine::{ActionId, DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn same_post_focus_replacement_drains_old_resources_before_new_generation_can_run() {
    let mut delivery = SegmentedDelivery::new(SegmentedCache::new());
    assert!(delivery.apply_focus(&focus(1, "https://old.example/root.m3u8")));
    let post = PostId::new("stream");
    delivery.active.insert(post.clone(), active());

    assert!(delivery.apply_focus(&focus(2, "https://new.example/root.m3u8")));
    assert!(delivery.active[&post].cancelling);
    assert_eq!(
        delivery.active_sources(),
        vec!["https://old.example/root.m3u8"]
    );
    assert_eq!(delivery.pending[&post].generation, 2);
    let finish = delivery
        .finish(done(post.clone()))
        .expect("old action keeps terminal ownership while draining");

    assert_eq!(
        finish.outcome,
        DecisionOutcome::Cancelled {
            bytes: 37,
            elapsed_ms: 25
        }
    );
    assert_eq!(
        finish.actual_resources,
        Some(ResourceCost::new(37, 0, 0, 1))
    );
    assert_eq!(delivery.pending[&post].generation, 2);
    assert_eq!(delivery.pending[&post].url, "https://new.example/root.m3u8");
    assert!(!delivery.active.contains_key(&post));
}

fn active() -> Active {
    let (cancellation, cancelled) = tokio::sync::oneshot::channel();
    Active {
        action: ActionId::new(7),
        fence: old_root_fence(),
        pending: Pending::root(1, 1, 0, "https://old.example/root.m3u8".to_owned()),
        committed_until_ms: u64::MAX,
        network: active_network(),
        _task: tokio::spawn(async move {
            let _ = cancelled.await;
            core::future::pending::<()>().await;
        }),
        cancellation: Some(cancellation),
        cancelling: false,
    }
}

fn done(post: PostId) -> SegmentedDone {
    SegmentedDone {
        action: ActionId::new(7),
        post,
        fence: old_root_fence(),
        outcome: Err(FetchFailure::cancelled(Some(telemetry()), 37)),
        observed_at_ms: 10,
        resources: Default::default(),
    }
}

fn telemetry() -> OriginTelemetry {
    OriginTelemetry {
        elapsed: core::time::Duration::from_millis(25),
        ttfb: None,
        concurrency: 1,
        network_class: NetworkClass::Wifi,
    }
}

fn focus(generation: u64, source: &str) -> DeliveryFocus {
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
        generation: FocusGeneration::try_new(generation).expect("valid test fixture"),
        transition: FocusTransition::RosterChange,
        rescue: None,
    }
}
