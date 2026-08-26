
use crate::delivery_events::{DeliveryFocus, DeliveryPlayback, FocusItem, FocusTransition, TransportRescue, TransportRescueReason};
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use core::time::Duration;

pub(super) fn focus(index: usize, watch_ms: u64, transition: FocusTransition) -> DeliveryFocus {
    let mut focus = DeliveryFocus::compatibility(
        ["a", "b", "c"].into_iter().map(item).collect(),
        index,
        watch_ms,
    );
    focus.transition = transition;
    focus
}

pub(super) fn rescue(index: usize, watch_ms: u64, reason: TransportRescueReason) -> DeliveryFocus {
    let mut focus = focus(index, watch_ms, FocusTransition::TransportRescue);
    focus.rescue = Some(TransportRescue {
        reason,
        rank_displacement: 1,
        wait_ms: 25,
    });
    focus
}

pub(super) fn empty(watch_ms: u64) -> DeliveryFocus {
    DeliveryFocus::compatibility(Vec::new(), 0, watch_ms)
}

pub(super) fn playback(
    id: &str,
    sequence: u64,
    position: u64,
    phase: PlaybackPhase,
) -> DeliveryPlayback {
    DeliveryPlayback {
        session: PlaybackSession::new(PostId::new(id), 1),
        sequence: PlaybackObservationSequence::new(sequence),
        observation: PlaybackObservation::try_new(
            Duration::from_millis(position),
            Duration::from_millis(position + 1_000),
            1_000,
            phase,
        )
        .expect("valid test fixture"),
    }
}

fn item(id: &str) -> FocusItem {
    FocusItem {
        post: PostId::new(id),
        meta: VideoMeta {
            urls: vec![format!("https://media.example/{id}.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(10_000),
            duration_ms: Some(10_000),
        },
    }
}
