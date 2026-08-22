use super::SegmentedDelivery;
use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::segmented::SegmentedCache;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn retained_old_generation_can_revive_after_a_later_focus_generation() {
    let cache = SegmentedCache::new();
    let mut delivery = SegmentedDelivery::new(cache.clone());
    let post = PostId::new("stream");
    assert!(delivery.apply_focus(&focus(1, &["a", "b"])));
    assert!(delivery.select_pending_root(&post, &root("b")));
    assert!(delivery.apply_focus(&focus(2, &["b", "a", "c"])));
    assert_eq!(delivery.pending[&post].generation, 1);
    delivery.pending.remove(&post);
    assert!(cache.mark_stage_failed(&post, 1, "retired".to_owned()));

    assert!(delivery.revive_root(&post, root("b")));
    assert_eq!(delivery.pending[&post].generation, 1);
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
