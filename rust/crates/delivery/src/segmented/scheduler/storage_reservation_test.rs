use super::progress::Pending;
use super::{Active, SegmentedDelivery};
use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::segmented::prepare::PreparedObject;
use crate::segmented::SegmentedCache;
use ghostr_engine::adaptive::HlsBootstrapStage;
use ghostr_engine::{ActionId, DeliveryKind, PostId, VideoMeta};
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
        assert!(cache.mark_stage_preparing(&held, 1, 500, bytes as u64));
        cache.store_stage_object(&held, 1, object(index, bytes));
    }
    assert!(cache.mark_stage_ready(&held, 1));
    let first = PostId::new("first");
    assert!(cache.mark_stage_preparing(
        &first,
        1,
        500,
        HlsBootstrapStage::Initialization.maximum_bytes(),
    ));
    delivery.active.insert(first, active("first"));
    delivery
        .pending
        .insert(PostId::new("second"), pending("second"));

    assert_eq!(delivery.available_bytes(), 7 * MIB as u64);
    delivery.cancel_all();
}

fn active(post: &str) -> Active {
    let (cancellation, _cancelled) = tokio::sync::oneshot::channel();
    Active {
        action: ActionId::new(1),
        pending: pending(post),
        committed_until_ms: u64::MAX,
        _task: tokio::spawn(std::future::pending()),
        cancellation: Some(cancellation),
        cancelling: false,
    }
}

fn pending(post: &str) -> Pending {
    Pending {
        generation: 1,
        source_index: 0,
        stage: HlsBootstrapStage::Initialization,
        url: format!("https://{post}.example/init.mp4"),
        after_init: Some(format!("https://{post}.example/first.m4s")),
    }
}

fn object(index: usize, bytes: usize) -> PreparedObject {
    let url = format!("https://first.example/{index}");
    PreparedObject {
        request_url: url.clone(),
        final_url: Url::parse(&url).unwrap(),
        body: Arc::from(vec![0; bytes]),
        content_type: None,
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
        generation: FocusGeneration::try_new(1).unwrap(),
        transition: FocusTransition::RosterChange,
        rescue: None,
    }
}
