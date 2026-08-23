use super::{RecoveryAction, SegmentedRetry};
use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::segmented::scheduler::{FailureDisposition, SegmentedDelivery};
use crate::segmented::SegmentedCache;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn same_cursor_retry_allocates_a_fresh_attempt_fence() {
    let mut delivery = SegmentedDelivery::new(SegmentedCache::new());
    delivery.apply_focus(&focus());
    let post = PostId::new("stream");
    let pending = delivery.pending.remove(&post).unwrap();
    let old_attempt = pending.attempt;
    let root = pending.root_source.clone();
    let retry = SegmentedRetry {
        post: post.clone(),
        pending,
        roots: vec![root],
        disposition: FailureDisposition::Requeue,
        detail: String::new(),
    };

    assert!(delivery.apply_recovery(retry, RecoveryAction::SameStage));
    assert!(delivery.pending[&post].attempt > old_attempt);
}

fn focus() -> DeliveryFocus {
    DeliveryFocus {
        items: vec![FocusItem {
            post: PostId::new("stream"),
            meta: VideoMeta {
                urls: vec!["https://media.example/root.m3u8".to_owned()],
                delivery: DeliveryKind::Hls,
                sha256: None,
                size_bytes: None,
                duration_ms: Some(4_000),
            },
        }],
        previews: Vec::new(),
        current_index: 0,
        watch_ms: 0,
        generation: FocusGeneration::try_new(1).unwrap(),
        transition: FocusTransition::RosterChange,
        rescue: None,
    }
}
