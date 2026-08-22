use crate::delivery_events::{DeliveryCandidate, DeliveryFocus, FocusItem};
use crate::manager::state::DeliveryState;
use ghostr_engine::{
    DataUsageLevel, DeliveryKind, EngineParams, PostId, PreviewDescriptor, VideoMeta,
};

pub(super) const BLURHASH: &str = "LEHV6nWB2yk8pyo0adR*.7kCMdnj";

pub(super) fn candidate_state() -> (DeliveryState, PostId, VideoMeta) {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    let post = PostId::new("preview");
    let meta = meta("https://media.example/preview.mp4");
    state.apply_candidate(DeliveryCandidate {
        post: post.clone(),
        meta: meta.clone(),
        preview: PreviewDescriptor::inline_blurhash(BLURHASH),
        metadata_evidence: Vec::new(),
        renditions: Vec::new(),
        discovered_at: 1,
    });
    (state, post, meta)
}

pub(super) fn focus(post: PostId, meta: VideoMeta) -> DeliveryFocus {
    DeliveryFocus::compatibility(vec![FocusItem { post, meta }], 0, 0)
}

pub(super) fn meta(source: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![source.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(1_000_000),
        duration_ms: Some(8_000),
    }
}
