use super::SegmentedDelivery;
use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::segmented::fetch::ObjectContinuation;
use crate::segmented::prepare::PreparedObject;
use crate::segmented::SegmentedCache;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_net::strong_etag::single_strong_etag;
use reqwest::header::{HeaderMap, HeaderValue, ETAG};
use std::sync::Arc;

#[test]
fn root_selection_preserves_same_cursor_and_clears_a_changed_root() {
    let cache = SegmentedCache::new();
    let mut delivery = SegmentedDelivery::new(cache.clone());
    delivery.apply_focus(&focus());
    let post = PostId::new("stream");
    assert!(cache.mark_stage_preparing(&post, 1, 500, 256 * 1024));
    assert!(matches!(
        cache.store_stage_block(
            &post,
            1,
            crate::segmented::cache::StageBlock::partial(0, object("a"))
        ),
        Some(crate::segmented::cache::StoredStage::Partial)
    ));
    let pending = delivery.pending.get_mut(&post).unwrap();
    *pending = pending.continued(continuation());
    let cursor = pending.cursor();

    assert!(delivery.select_pending_root(&post, &root("a")));
    assert_eq!(delivery.pending[&post].cursor(), cursor);
    assert!(delivery.select_pending_root(&post, &root("b")));
    assert_eq!(cache.snapshot("stream").bytes_present, 0);
    assert_ne!(delivery.pending[&post].attempt, cursor.attempt);
    assert_eq!(delivery.pending[&post].cursor().next_offset, 0);
}

fn continuation() -> ObjectContinuation {
    let mut headers = HeaderMap::new();
    headers.insert(ETAG, HeaderValue::from_static("\"v1\""));
    ObjectContinuation {
        next_offset: 256 * 1024,
        total: 512 * 1024,
        final_url: root("a").parse().unwrap(),
        strong_etag: single_strong_etag(&headers).unwrap().unwrap(),
    }
}

fn object(name: &str) -> PreparedObject {
    PreparedObject {
        request_url: root(name),
        final_url: root(name).parse().unwrap(),
        body: Arc::from(vec![1; 256 * 1024]),
        content_type: None,
        cache: Default::default(),
    }
}

fn focus() -> DeliveryFocus {
    DeliveryFocus {
        items: vec![FocusItem {
            post: PostId::new("stream"),
            meta: VideoMeta {
                urls: vec![root("a"), root("b")],
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

fn root(name: &str) -> String {
    format!("https://{name}.example/root.m3u8")
}
