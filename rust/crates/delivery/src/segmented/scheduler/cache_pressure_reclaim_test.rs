use super::SegmentedDelivery;
use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::segmented::prepare::PreparedObject;
use crate::segmented::{SegmentedCache, SegmentedPhase};
use ghostr_engine::adaptive::HlsBootstrapStage;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;

pub(super) const MIB: usize = 1024 * 1024;

#[test]
fn current_shift_reclaims_only_unprotected_ready_bootstrap_under_pressure() {
    let cache = SegmentedCache::new();
    let mut delivery = SegmentedDelivery::new(cache.clone());
    delivery.apply_focus(&focus(1, 0));
    store(&cache, "first", 1, &[MIB, MIB, 8 * MIB, 8 * MIB]);
    assert!(cache.mark_stage_ready(&PostId::new("first"), 1));
    store(&cache, "second", 1, &[MIB, MIB, 8 * MIB]);

    delivery.apply_focus(&focus(2, 1));
    assert_eq!(cache.snapshot("first").phase, SegmentedPhase::Ready);
    store(&cache, "second", 2, &[MIB, MIB, 8 * MIB]);
    assert_eq!(cache.physical_available_bytes(), 4 * MIB as u64);
    assert_eq!(delivery.available_bytes(), 22 * MIB as u64);
    assert!(cache.mark_stage_preparing(
        &PostId::new("second"),
        2,
        500,
        HlsBootstrapStage::FirstSegment.maximum_bytes(),
    ));

    assert_eq!(cache.snapshot("first").phase, SegmentedPhase::Queued);
    assert_eq!(cache.snapshot("second").bytes_present, 10 * MIB as u64);
}

pub(super) fn store(cache: &SegmentedCache, post: &str, generation: u64, sizes: &[usize]) {
    let post = PostId::new(post);
    for (index, bytes) in sizes.iter().copied().enumerate() {
        assert!(cache.mark_stage_preparing(&post, generation, 500, bytes as u64));
        assert!(cache
            .store_stage_object(&post, generation, object(post.as_str(), index, bytes))
            .is_some());
    }
}

fn object(post: &str, index: usize, bytes: usize) -> PreparedObject {
    let url = format!("https://{post}.example/{index}");
    PreparedObject {
        request_url: url.clone(),
        final_url: url.parse().unwrap(),
        body: Arc::from(vec![0; bytes]),
        content_type: None,
    }
}

pub(super) fn focus(generation: u64, current_index: usize) -> DeliveryFocus {
    DeliveryFocus {
        items: ["first", "second"]
            .into_iter()
            .map(|post| FocusItem {
                post: PostId::new(post),
                meta: VideoMeta {
                    urls: vec![format!("https://{post}.example/root.m3u8")],
                    delivery: DeliveryKind::Hls,
                    sha256: None,
                    size_bytes: None,
                    duration_ms: Some(4_000),
                },
            })
            .collect(),
        previews: Vec::new(),
        current_index,
        watch_ms: 0,
        generation: FocusGeneration::try_new(generation).unwrap(),
        transition: FocusTransition::UserNavigation,
        rescue: None,
    }
}
