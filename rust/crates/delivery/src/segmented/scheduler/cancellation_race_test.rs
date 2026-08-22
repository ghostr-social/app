use super::progress::Pending;
use super::{Active, SegmentedDelivery, SegmentedDone};
use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::segmented::fetch::{FetchFailure, OriginTelemetry};
use crate::segmented::SegmentedCache;
use ghostr_engine::adaptive::DecisionOutcome;
use ghostr_engine::origin_model::{ErrorReason, NetworkClass};
use ghostr_engine::{ActionId, DeliveryKind, PostId, VideoMeta};
use std::time::Duration;

#[tokio::test]
async fn queued_physical_terminal_wins_over_a_late_focus_cancellation() {
    let mut delivery = SegmentedDelivery::new(SegmentedCache::new());
    delivery.apply_focus(&focus(1, "https://old.example/root.m3u8"));
    let post = PostId::new("stream");
    delivery.active.insert(post.clone(), completed_active());

    delivery.apply_focus(&focus(2, "https://new.example/root.m3u8"));
    assert!(!delivery.active[&post].cancelling);
    let finish = delivery
        .finish(failed(post))
        .expect("queued terminal remains owned");

    assert!(matches!(
        finish.outcome,
        DecisionOutcome::Failed { ref class, .. } if class == "warp_hls_http_5xx"
    ));
    assert_eq!(
        finish.observation.unwrap().outcome,
        ghostr_engine::origin_model::OriginOutcome::Failure(ErrorReason::Http5xx)
    );
}

fn completed_active() -> Active {
    let (cancellation, cancelled) = tokio::sync::oneshot::channel();
    drop(cancelled);
    Active {
        action: ActionId::new(7),
        pending: Pending::root(1, 0, "https://old.example/root.m3u8".to_owned()),
        committed_until_ms: u64::MAX,
        _task: tokio::spawn(std::future::pending()),
        cancellation: Some(cancellation),
        cancelling: false,
    }
}

fn failed(post: PostId) -> SegmentedDone {
    SegmentedDone {
        action: ActionId::new(7),
        post,
        generation: 1,
        outcome: Err(FetchFailure::admitted(
            anyhow::anyhow!("server"),
            ErrorReason::Http5xx,
            telemetry(),
            0,
        )),
        observed_at_ms: 10,
        resources: Default::default(),
    }
}

fn telemetry() -> OriginTelemetry {
    OriginTelemetry {
        elapsed: Duration::from_millis(25),
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
        generation: FocusGeneration::try_new(generation).unwrap(),
        transition: FocusTransition::RosterChange,
        rescue: None,
    }
}
