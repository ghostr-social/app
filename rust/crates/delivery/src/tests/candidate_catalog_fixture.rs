use crate::delivery_events::DeliveryCandidate;
use crate::manager::state::DeliveryState;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

pub(super) fn binding(state: &mut DeliveryState, post: &str) -> RepresentationBinding {
    state
        .take_representation_bindings()
        .into_iter()
        .find(|binding| binding.post() == &PostId::new(post))
        .expect("candidate binding")
}

pub(super) fn candidate(id: &str, discovered_at: u64) -> DeliveryCandidate {
    DeliveryCandidate {
        post: PostId::new(id),
        meta: VideoMeta {
            urls: vec![url(id)],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(16),
            duration_ms: Some(1_000),
        },
        metadata_evidence: Vec::new(),
        renditions: Vec::new(),
        discovered_at,
    }
}

pub(super) fn url(id: &str) -> String {
    format!("https://media.example/{id}.mp4")
}
