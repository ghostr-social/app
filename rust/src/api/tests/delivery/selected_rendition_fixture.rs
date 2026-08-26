use core::time::Duration;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::playback::{
    AdaptiveBufferPolicy, EstimateConfidence, MediaConsumption, NetworkConditions,
    PlaybackObservation, PlaybackPhase,
};
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

pub(super) struct SelectedRendition {
    pub(super) advertised: VideoMeta,
    pub(super) advertised_representation: String,
    pub(super) selected: VideoMeta,
    pub(super) binding: RepresentationBinding,
}

pub(super) fn selected_rendition(post: &str) -> SelectedRendition {
    let advertised = meta("high", 64);
    let selected = meta("low", 16);
    let variants = vec![
        variant(advertised.clone(), 6_000_000),
        variant(selected.clone(), 1_000_000),
    ];
    let mut catalog = Catalog::new();
    let source = catalog.upsert_with_renditions(PostId::new(post), advertised.clone(), variants);
    let (network, observation, target) = stalled_selection();
    let binding = catalog
        .select_rendition(&PostId::new(post), network, observation, target)
        .expect("lower rendition selected");
    SelectedRendition {
        advertised,
        advertised_representation: source.representation().fingerprint().to_owned(),
        selected,
        binding,
    }
}

fn stalled_selection() -> (
    NetworkConditions,
    PlaybackObservation,
    ghostr_engine::playback::BufferTarget,
) {
    let network = NetworkConditions::new(
        250_000,
        0,
        Duration::from_millis(100),
        EstimateConfidence::High,
    );
    let observation = PlaybackObservation::try_new(
        Duration::ZERO,
        Duration::from_secs(1),
        1_000,
        PlaybackPhase::NetworkStalled,
    )
    .expect("test fixture precondition must hold");
    let target =
        AdaptiveBufferPolicy::default().target(network, MediaConsumption::new(6_000_000, 1_000));
    (network, observation, target)
}

fn variant(meta: VideoMeta, bitrate: u64) -> VideoRendition {
    VideoRendition::try_new(meta, Some(bitrate)).expect("test fixture precondition must hold")
}

fn meta(name: &str, size_bytes: u64) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://{name}.example/video.mp4")],
        delivery: DeliveryKind::Progressive,
        sha256: Some(format!("{name}-digest")),
        size_bytes: Some(size_bytes),
        duration_ms: Some(2_000),
    }
}
