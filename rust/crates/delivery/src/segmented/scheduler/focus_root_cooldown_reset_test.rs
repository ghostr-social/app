use super::SegmentedDelivery;
use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::segmented::SegmentedCache;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn adding_an_untried_root_resets_the_selected_roots_strict_cooldown() {
    let mut delivery = SegmentedDelivery::new(SegmentedCache::new());
    assert!(delivery.apply_focus(&focus(1, &["a", "b"])));
    let post = PostId::new("stream");
    assert!(delivery.select_pending_root(&post, &root("b")));

    assert_eq!(
        delivery.hls_cooldown_resets(&focus(2, &["b", "a", "c"])),
        vec![post.clone()]
    );
    assert_eq!(
        delivery.hls_cooldown_resets(&focus(3, &["a", "c"])),
        vec![post]
    );
}

#[test]
fn adding_an_untried_root_resets_cooldown_after_all_roots_retire() {
    let cache = SegmentedCache::new();
    let mut delivery = SegmentedDelivery::new(cache.clone());
    assert!(delivery.apply_focus(&focus(1, &["a", "b"])));
    let post = PostId::new("stream");
    let pending = delivery.pending.remove(&post).unwrap();
    assert!(cache.mark_stage_failed(&post, pending.generation, "retired".to_owned()));

    assert!(delivery
        .hls_cooldown_resets(&focus(2, &["b", "a"]))
        .is_empty());
    assert_eq!(
        delivery.hls_cooldown_resets(&focus(3, &["a", "b", "c"])),
        vec![post]
    );
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
        generation: FocusGeneration::try_new(generation).unwrap(),
        transition: FocusTransition::RosterChange,
        rescue: None,
    }
}

fn root(name: &str) -> String {
    format!("https://{name}.example/root.m3u8")
}
