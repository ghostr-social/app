use crate::playback_demand::{ConsumerId, DemandLease, DemandState};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{ByteRange, DeliveryKind, PostId, VideoMeta};

pub(super) fn catalog(posts: &[&str]) -> Catalog {
    let mut catalog = Catalog::new();
    for post in posts {
        catalog.upsert(PostId::new(*post), meta(post));
    }
    catalog
}

pub(super) fn binding(catalog: &Catalog, post: &str) -> RepresentationBinding {
    catalog.binding(&PostId::new(post)).expect("binding")
}

pub(super) fn blocked(
    id: u64,
    post: &str,
    binding: RepresentationBinding,
    range: ByteRange,
) -> DemandState {
    DemandState::Blocked(DemandLease::new(
        ConsumerId::new(id).expect("consumer ID"),
        PostId::new(post),
        Some(binding),
        range,
    ))
}

pub(super) fn meta(tag: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://{tag}.example/video.mp4")],
        delivery: DeliveryKind::Progressive,
        sha256: Some(format!("{tag}-digest")),
        size_bytes: Some(1_000),
        duration_ms: Some(1_000),
    }
}
