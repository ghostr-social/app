use super::SegmentedDelivery;
use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::segmented::SegmentedCache;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn retained_post_delivery_transition_is_a_representation_change() {
    let mut delivery = SegmentedDelivery::new(SegmentedCache::new());
    assert!(delivery.apply_focus(&focus(1, DeliveryKind::Hls)));

    let changed = delivery.changed_hls_sources(&focus(2, DeliveryKind::Progressive));

    assert_eq!(changed, vec![PostId::new("stream")]);
}

fn focus(generation: u64, delivery: DeliveryKind) -> DeliveryFocus {
    DeliveryFocus {
        items: vec![FocusItem {
            post: PostId::new("stream"),
            meta: VideoMeta {
                urls: vec!["https://media.example/video".to_owned()],
                delivery,
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
