use crate::delivery_fixture::media::{hit_log, media_body, serve_recording};
use ghostr_delivery::delivery_events::{
    PlayerPreparationAttempt, PlayerPreparationAuthority, PlayerPreparationObservation,
    PlayerPreparationReport, PlayerPreparationState,
};
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::{DeliveryKind, VideoMeta};
use ghostr_partial_store::partial_range_store::ContentRevision;
use sha2::{Digest as _, Sha256};

pub(super) fn initializing(
    binding: RepresentationBinding,
    revision: ContentRevision,
) -> PlayerPreparationReport {
    let post = binding.post().clone();
    PlayerPreparationReport::try_new(
        PlayerPreparationAuthority::try_new(post, binding, revision, "asset")
            .expect("valid test fixture"),
        PlayerPreparationAttempt::try_new(7, 1, 1).expect("valid test fixture"),
        1,
        PlayerPreparationObservation::try_new(PlayerPreparationState::Initializing, None, 1)
            .expect("valid test fixture"),
    )
    .expect("valid test fixture")
}

pub(super) fn rendition(id: &str) -> VideoRendition {
    rendition_at(
        format!("https://media.example/{id}.mp4"),
        if id == "high" { "a" } else { "b" }.repeat(64),
    )
}

pub(super) async fn verified_rendition(id: &str) -> VideoRendition {
    let bytes = media_body();
    let digest = format!("{:x}", Sha256::digest(&bytes));
    let url = serve_recording(id, bytes, hit_log()).await;
    rendition_at(url, digest)
}

fn rendition_at(url: String, digest: String) -> VideoRendition {
    VideoRendition::try_new(
        VideoMeta {
            urls: vec![url],
            delivery: DeliveryKind::Progressive,
            sha256: Some(digest),
            size_bytes: Some(16),
            duration_ms: Some(2_000),
        },
        None,
    )
    .expect("valid test fixture")
}
