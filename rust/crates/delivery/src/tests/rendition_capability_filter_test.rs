
use crate::delivery_events::{DeliveryCandidate, DeliveryPlayback, PlayerPreparationAttempt, PlayerPreparationAuthority, PlayerPreparationObservation, PlayerPreparationReport, PlayerPreparationState, DECODER_UNSUPPORTED_FAILURE};
use crate::manager::quality::select_rendition;
use crate::tests::player_preparation_fixture::{meta, state};
use ghostr_engine::host_stats::{HostStats, ThroughputSample};
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;
use core::time::Duration;

#[test]
fn quality_selection_never_reselects_decoder_blocked_rendition() {
    let mut state = state(&["adaptive"], 0);
    let post = PostId::new("adaptive");
    let high = rendition("high", 6_000_000);
    let low = rendition("low", 1_000_000);
    state.apply_candidate(DeliveryCandidate {
        post: post.clone(),
        meta: high.meta().clone(),
        preview: None,
        metadata_evidence: Vec::new(),
        renditions: vec![high, low],
        discovered_at: 1,
    });
    state.take_representation_bindings();
    evidence(&mut state, 1, PlayerPreparationState::Initializing, None);
    evidence(
        &mut state,
        2,
        PlayerPreparationState::Failed,
        Some(DECODER_UNSUPPORTED_FAILURE),
    );
    state.select_capability_fallback(&post, 0).expect("valid test fixture");
    assert!(state.apply_playback(&playback(post)).is_accepted());

    assert!(select_rendition(&mut state, &fast_stats(), 8_000).is_none());
    assert_eq!(
        state
            .catalog()
            .lookup(&PostId::new("adaptive"))
            .expect("valid test fixture")
            .meta,
        meta("low")
    );
}

fn evidence(
    state: &mut crate::manager::state::DeliveryState,
    sequence: u64,
    status: PlayerPreparationState,
    failure: Option<&str>,
) {
    let post = PostId::new("adaptive");
    let authority = PlayerPreparationAuthority::try_new(
        post.clone(),
        state.catalog().binding(&post).expect("valid test fixture"),
        ContentRevision::default(),
        "asset",
    )
    .expect("valid test fixture");
    let observation =
        PlayerPreparationObservation::try_new(status, failure.map(str::to_owned), sequence)
            .expect("valid test fixture");
    let report = PlayerPreparationReport::try_new(
        authority,
        PlayerPreparationAttempt::try_new(7, 1, 1).expect("valid test fixture"),
        sequence,
        observation,
    )
    .expect("valid test fixture");
    assert!(state.apply_player_preparation(report));
}

fn playback(post: PostId) -> DeliveryPlayback {
    DeliveryPlayback {
        session: PlaybackSession::new(post, 1),
        sequence: PlaybackObservationSequence::new(1),
        observation: PlaybackObservation::try_new(
            Duration::ZERO,
            Duration::from_secs(10),
            1_000,
            PlaybackPhase::Playing,
        )
        .expect("valid test fixture"),
    }
}

fn fast_stats() -> HostStats {
    let mut stats = HostStats::new();
    for second in 1..=8 {
        stats.record_host_throughput(
            "media.example",
            ThroughputSample::new(3_000_000, Duration::from_secs(1), second * 1_000, 1).expect("valid test fixture"),
        );
    }
    stats
}

fn rendition(id: &str, bitrate: u64) -> VideoRendition {
    VideoRendition::try_new(meta(id), Some(bitrate)).expect("valid test fixture")
}
