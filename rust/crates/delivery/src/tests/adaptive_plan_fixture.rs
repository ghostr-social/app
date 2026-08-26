use crate::delivery_events::{DeliveryFocus, DeliveryPlayback, FocusItem};
use crate::manager::state::DeliveryState;
use ghostr_engine::catalog::LearnedFacts;
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use core::time::Duration;

pub(super) fn state() -> DeliveryState {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_focus(
        DeliveryFocus::compatibility((0..12).map(item).collect(), 0, 0),
        0,
    );
    for index in 0..12 {
        state.catalog_mut().learn(
            &PostId::new(format!("p{index}")),
            LearnedFacts {
                accept_ranges: Some(true),
                ..LearnedFacts::default()
            },
        );
    }
    state
}

pub(super) fn playback_for(post: PostId, buffer_ms: u64) -> DeliveryPlayback {
    DeliveryPlayback {
        session: PlaybackSession::new(post, 1),
        sequence: PlaybackObservationSequence::new(1),
        observation: PlaybackObservation::try_new(
            Duration::ZERO,
            Duration::from_millis(buffer_ms),
            1_000,
            PlaybackPhase::Playing,
        )
        .expect("valid test fixture"),
    }
}

pub(super) fn source(index: usize) -> String {
    format!("https://media.example/p{index}.mp4")
}

pub(super) fn refocus(state: &mut DeliveryState, current_index: usize, observed_at_ms: u64) {
    state.apply_focus(
        DeliveryFocus::compatibility((0..12).map(item).collect(), current_index, 0),
        observed_at_ms,
    );
}

fn item(index: usize) -> FocusItem {
    FocusItem {
        post: PostId::new(format!("p{index}")),
        meta: VideoMeta {
            urls: vec![source(index)],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(1_000_000),
            duration_ms: Some(8_000),
        },
    }
}
