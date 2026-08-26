use ghostr_delivery::delivery_events::{
    PlayerPreparationAttempt, PlayerPreparationAuthority, PlayerPreparationObservation,
    PlayerPreparationReport, PlayerPreparationState,
};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::ContentRevision;

pub(super) fn initializing(post: &PostId, meta: &VideoMeta) -> PlayerPreparationReport {
    let binding = Catalog::new().upsert(post.clone(), meta.clone());
    PlayerPreparationReport::try_new(
        PlayerPreparationAuthority::try_new(
            post.clone(),
            binding,
            ContentRevision::default(),
            "asset",
        )
        .expect("valid test fixture"),
        PlayerPreparationAttempt::try_new(7, 1, 1).expect("valid test fixture"),
        1,
        PlayerPreparationObservation::try_new(PlayerPreparationState::Initializing, None, 1)
            .expect("valid test fixture"),
    )
    .expect("valid test fixture")
}

pub(super) fn rendition(id: &str) -> VideoRendition {
    VideoRendition::try_new(
        VideoMeta {
            urls: vec![format!("https://media.example/{id}.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: Some(if id == "high" { "a" } else { "b" }.repeat(64)),
            size_bytes: Some(16),
            duration_ms: Some(2_000),
        },
        None,
    )
    .expect("valid test fixture")
}
