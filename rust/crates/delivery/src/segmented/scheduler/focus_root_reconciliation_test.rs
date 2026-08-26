use super::progress::Pending;
use super::{active_network, test_fence, Active, SegmentedDelivery};
use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::segmented::{SegmentedCache, SegmentedPhase};
use ghostr_engine::{ActionId, DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn roster_edit_preserves_an_active_unchanged_root_and_generation() {
    let cache = SegmentedCache::new();
    let mut delivery = SegmentedDelivery::new(cache.clone());
    assert!(delivery.apply_focus(&focus(1, &["a", "b"])));
    let post = PostId::new("stream");
    assert!(cache.mark_stage_preparing(&post, 1, 500, root_maximum()));
    delivery.active.insert(post.clone(), active());

    assert!(delivery.apply_focus(&focus(2, &["b", "a", "c"])));

    let active = &delivery.active[&post];
    assert!(!active.cancelling);
    assert_eq!(active.pending.generation, 1);
    assert_eq!(active.pending.source_index, 0);
    assert_eq!(cache.snapshot("stream").phase, SegmentedPhase::Preparing);
    delivery
        .active
        .remove(&post)
        .expect("valid test fixture")
        ._task
        .abort();
}

fn active() -> Active {
    let (cancellation, cancelled) = tokio::sync::oneshot::channel();
    Active {
        action: ActionId::new(7),
        fence: test_fence(1, 1, &root("b"), root_maximum()),
        pending: Pending::root(1, 1, 1, root("b")),
        committed_until_ms: u64::MAX,
        network: active_network(),
        _task: tokio::spawn(async move {
            let _ = cancelled.await;
        }),
        cancellation: Some(cancellation),
        cancelling: false,
    }
}

fn focus(generation: u64, names: &[&str]) -> DeliveryFocus {
    DeliveryFocus {
        items: vec![FocusItem {
            post: PostId::new("stream"),
            meta: VideoMeta {
                urls: names.iter().map(|name| root(name)).collect(),
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

fn root(name: &str) -> String {
    format!("https://{name}.example/root.m3u8")
}

fn root_maximum() -> u64 {
    ghostr_engine::adaptive::HlsBootstrapStage::RootManifest.maximum_bytes()
}
