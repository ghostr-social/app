use super::SegmentedDelivery;
use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::segmented::SegmentedCache;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn reordering_the_same_hls_roots_is_not_a_representation_change() {
    let mut delivery = SegmentedDelivery::new(SegmentedCache::new());
    assert!(delivery.apply_focus(&focus(1, &["a", "b"])));

    let changed = delivery.changed_hls_sources(&focus(2, &["b", "a"]));

    assert!(changed.is_empty());
}

fn focus(generation: u64, roots: &[&str]) -> DeliveryFocus {
    DeliveryFocus {
        items: vec![FocusItem {
            post: PostId::new("stream"),
            meta: VideoMeta {
                urls: roots
                    .iter()
                    .map(|root| format!("https://{root}.example/root.m3u8"))
                    .collect(),
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
