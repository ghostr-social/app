use super::progress::Pending;
use super::{Active, SegmentedDelivery};
use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::segmented::{SegmentedCache, SegmentedPhase};
use ghostr_engine::{ActionId, DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn equivalent_focus_keeps_inflight_hls_bootstrap() {
    let cache = SegmentedCache::new();
    let mut delivery = SegmentedDelivery::new(cache.clone());
    assert!(delivery.apply_focus(&focus(1, SOURCE)));
    let post = PostId::new("stream");
    assert!(cache.mark_stage_preparing(&post, 1, 500, root_maximum()));
    delivery.active.insert(post.clone(), active());

    let mut refreshed = focus(2, SOURCE);
    refreshed.watch_ms = 1_500;
    refreshed.transition = FocusTransition::UserNavigation;
    refreshed.items[0].meta.duration_ms = Some(5_000);
    assert!(!delivery.apply_focus(&refreshed));

    assert!(delivery.active.contains_key(&post));
    assert_eq!(delivery.active[&post].pending.generation, 1);
    assert_eq!(cache.snapshot("stream").phase, SegmentedPhase::Preparing);
    assert!(cache.mark_stage_preparing(&post, 1, 700, root_maximum()));
    assert_eq!(cache.snapshot("stream").eta_ms, Some(700));
    delivery.cancel_all();
    assert!(delivery.active[&post].cancelling);
    delivery.active.remove(&post).unwrap()._task.abort();
}

#[tokio::test]
async fn changed_hls_source_cancels_inflight_bootstrap() {
    let cache = SegmentedCache::new();
    let mut delivery = SegmentedDelivery::new(cache.clone());
    assert!(delivery.apply_focus(&focus(1, SOURCE)));
    let post = PostId::new("stream");
    assert!(cache.mark_stage_preparing(&post, 1, 500, root_maximum()));
    delivery.active.insert(post.clone(), active());

    assert!(delivery.apply_focus(&focus(2, "https://backup.example/index.m3u8")));

    assert!(delivery.active[&post].cancelling);
    assert_eq!(cache.snapshot("stream").phase, SegmentedPhase::Queued);
    delivery.active.remove(&post).unwrap()._task.abort();
}

fn active() -> Active {
    let (cancellation, cancelled) = tokio::sync::oneshot::channel();
    Active {
        action: ActionId::new(1),
        pending: Pending::root(1, 0, SOURCE.to_owned()),
        committed_until_ms: u64::MAX,
        _task: tokio::spawn(async move {
            let _ = cancelled.await;
        }),
        cancellation: Some(cancellation),
        cancelling: false,
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

const SOURCE: &str = "https://media.example/index.m3u8";

fn root_maximum() -> u64 {
    ghostr_engine::adaptive::HlsBootstrapStage::RootManifest.maximum_bytes()
}
