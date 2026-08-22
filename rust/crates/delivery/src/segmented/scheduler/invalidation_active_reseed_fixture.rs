use super::{progress::Pending, Active, SegmentedDone};
use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::segmented::fetch::FetchFailure;
use crate::segmented::prepare::PreparedObject;
use crate::segmented::{CachedHlsGeneration, SegmentedCache};
use ghostr_engine::adaptive::HlsBootstrapStage;
use ghostr_engine::origin_model::ErrorReason;
use ghostr_engine::{ActionId, DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;

pub(super) fn ready_root(cache: &SegmentedCache, post: &PostId) -> CachedHlsGeneration {
    let source = root("stream");
    let body: Arc<[u8]> = Arc::from(b"#EXTM3U\n#EXTINF:4,\nsegment.m4s\n".as_slice());
    assert!(cache.mark_stage_preparing(post, 1, 1, body.len() as u64));
    cache
        .store_stage_object(
            post,
            1,
            PreparedObject {
                request_url: source.clone(),
                final_url: source.parse().unwrap(),
                body,
                content_type: None,
                cache: Default::default(),
            },
        )
        .unwrap();
    assert!(cache.mark_stage_ready(post, 1));
    cache.object(&source).unwrap().generation()
}

pub(super) fn active() -> Active {
    let (cancellation, cancelled) = tokio::sync::oneshot::channel();
    let mut pending = Pending::root(1, 1, 0, root("stream"));
    pending.stage = HlsBootstrapStage::Initialization;
    pending.url = root("init");
    Active {
        action: ActionId::new(7),
        pending,
        committed_until_ms: u64::MAX,
        _task: tokio::spawn(async move {
            let _ = cancelled.await;
        }),
        cancellation: Some(cancellation),
        cancelling: false,
    }
}

pub(super) fn cancelled(post: PostId) -> SegmentedDone {
    SegmentedDone {
        action: ActionId::new(7),
        post,
        generation: 1,
        outcome: Err(FetchFailure::preflight(
            anyhow::anyhow!("late"),
            ErrorReason::Policy,
        )),
        observed_at_ms: 1,
        resources: Default::default(),
    }
}

pub(super) fn focus() -> DeliveryFocus {
    DeliveryFocus {
        items: ["stream", "other"].into_iter().map(item).collect(),
        previews: Vec::new(),
        current_index: 0,
        watch_ms: 0,
        generation: FocusGeneration::try_new(1).unwrap(),
        transition: FocusTransition::RosterChange,
        rescue: None,
    }
}

fn item(name: &str) -> FocusItem {
    FocusItem {
        post: PostId::new(name),
        meta: VideoMeta {
            urls: vec![root(name)],
            delivery: DeliveryKind::Hls,
            sha256: None,
            size_bytes: None,
            duration_ms: Some(4_000),
        },
    }
}

pub(super) fn root(name: &str) -> String {
    format!("https://{name}.example/root.m3u8")
}
