use super::storage_reservation_fixture::{active, pending, reserve_active, store_complete};
use super::SegmentedDelivery;
use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::segmented::prepare::PreparedObject;
use crate::segmented::SegmentedCache;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use url::Url;

const MIB: usize = 1024 * 1024;

#[tokio::test]
async fn active_stage_reservation_prevents_a_second_stage_overcommit() {
    let cache = SegmentedCache::new();
    let mut delivery = SegmentedDelivery::new(cache.clone());
    delivery.apply_focus(&focus());
    let held = PostId::new("held");
    for (index, bytes) in [8 * MIB, 8 * MIB, MIB].into_iter().enumerate() {
        store_complete(&cache, &held, index as u64 + 1, object(index, bytes));
    }
    assert!(cache.mark_stage_ready(&held, 1));
    let first = PostId::new("first");
    let first_active = active("first");
    let lease = reserve_active(&cache, &first, &first_active);
    delivery.active.insert(first, first_active);
    delivery
        .pending
        .insert(PostId::new("second"), pending("second"));

    assert_eq!(delivery.available_bytes(), 7 * MIB as u64);
    delivery.cancel_all();
    drop(lease);
}

fn object(index: usize, bytes: usize) -> PreparedObject {
    let url = format!("https://first.example/{index}");
    PreparedObject {
        request_url: url.clone(),
        final_url: Url::parse(&url).expect("valid test fixture"),
        body: Arc::from(vec![0; bytes]),
        content_type: None,
        cache: Default::default(),
    }
}

fn focus() -> DeliveryFocus {
    DeliveryFocus {
        items: ["first", "second", "held"]
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
        current_index: 0,
        watch_ms: 0,
        generation: FocusGeneration::try_new(1).expect("valid test fixture"),
        transition: FocusTransition::RosterChange,
        rescue: None,
    }
}
