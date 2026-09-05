use crate::partial_range_store::CompiledIndexKey;
use ghostr_engine::media_timeline::{parse_mp4_segments_with_control, MediaSegment, MediaTimeline};
use ghostr_engine::representation::{RepresentationId, SourceGeneration};
use ghostr_engine::{DeliveryKind, VideoMeta};

pub(super) fn timeline() -> MediaTimeline {
    let words: [u32; 11] = [44, 0x7369_6478, 0, 1, 1_000, 0, 0, 1, 8, 1_000, 0x9000_0000];
    let bytes: Vec<u8> = words.into_iter().flat_map(u32::to_be_bytes).collect();
    parse_mp4_segments_with_control(
        &[MediaSegment::new(0, &bytes)],
        &core::sync::atomic::AtomicBool::new(false),
    )
    .expect("fixture")
}

pub(super) fn key(etag: &str) -> CompiledIndexKey {
    let source =
        SourceGeneration::try_new("https://media.example/clip", etag, 52).expect("fixture");
    let representation = RepresentationId::for_meta(&VideoMeta {
        urls: vec![source.final_url().to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(52),
        duration_ms: Some(1_000),
    });
    CompiledIndexKey::native_mp4(&representation, &source)
}
