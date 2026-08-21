use super::SegmentedDelivery;
use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::segmented::SegmentedCache;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn hls_preparation_priority_preserves_its_true_feed_offset() {
    let mut delivery = SegmentedDelivery::new(SegmentedCache::new());
    delivery.apply_focus(&focus());

    assert_eq!(
        delivery.targets[0].priority,
        PreemptionAuthority::Transition
    );
    assert_eq!(
        delivery.targets[1].priority,
        PreemptionAuthority::Speculative
    );
}

fn focus() -> DeliveryFocus {
    DeliveryFocus {
        items: vec![
            item("current", DeliveryKind::Progressive),
            item("next", DeliveryKind::Hls),
            item("far", DeliveryKind::Hls),
        ],
        current_index: 0,
        watch_ms: 0,
        generation: FocusGeneration::try_new(1).unwrap(),
        transition: FocusTransition::RosterChange,
        rescue: None,
    }
}

fn item(post: &str, delivery: DeliveryKind) -> FocusItem {
    FocusItem {
        post: PostId::new(post),
        meta: VideoMeta {
            urls: vec![format!("https://{post}.example/index.m3u8")],
            delivery,
            sha256: None,
            size_bytes: None,
            duration_ms: Some(4_000),
        },
    }
}
