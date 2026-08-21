use super::{Active, SegmentedDelivery};
use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::segmented::{SegmentedCache, SegmentedPhase};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn equivalent_focus_keeps_inflight_hls_bootstrap() {
    let cache = SegmentedCache::new();
    let mut delivery = SegmentedDelivery::new(cache.clone());
    delivery.apply_focus(&focus(1, SOURCE));
    let post = PostId::new("stream");
    assert!(cache.mark_preparing(&post, 1, 500));
    delivery.active.insert(
        post.clone(),
        Active {
            generation: 1,
            task: tokio::spawn(std::future::pending()),
        },
    );

    let mut refreshed = focus(2, SOURCE);
    refreshed.watch_ms = 1_500;
    refreshed.transition = FocusTransition::UserNavigation;
    refreshed.items[0].meta.duration_ms = Some(5_000);
    delivery.apply_focus(&refreshed);

    assert!(delivery.active.contains_key(&post));
    assert_eq!(delivery.active[&post].generation, 1);
    assert_eq!(cache.snapshot("stream").phase, SegmentedPhase::Preparing);
    assert!(cache.mark_preparing(&post, 1, 700));
    assert_eq!(cache.snapshot("stream").eta_ms, Some(700));
    delivery.abort_all();
}

#[tokio::test]
async fn changed_hls_source_cancels_inflight_bootstrap() {
    let cache = SegmentedCache::new();
    let mut delivery = SegmentedDelivery::new(cache.clone());
    delivery.apply_focus(&focus(1, SOURCE));
    let post = PostId::new("stream");
    assert!(cache.mark_preparing(&post, 1, 500));
    delivery.active.insert(
        post.clone(),
        Active {
            generation: 1,
            task: tokio::spawn(std::future::pending()),
        },
    );

    delivery.apply_focus(&focus(2, "https://backup.example/index.m3u8"));

    assert!(!delivery.active.contains_key(&post));
    assert_eq!(cache.snapshot("stream").phase, SegmentedPhase::Queued);
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
