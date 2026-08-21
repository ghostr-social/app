use crate::delivery_events::{DeliveryFocus, FocusItem};
use crate::manager::plan::PlannedWork;
use crate::manager::state::DeliveryState;
use crate::tests::adaptive_plan_fixture::playback_for;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};

mod active;
mod evidence;
mod stats;

pub(super) const PRIMARY: &str = "https://slow.example/video.mp4";
pub(super) const ALTERNATE: &str = "https://fast.example/video.mp4";
pub(super) const PRIMARY_ACTION_ID: u64 = 1;
pub(super) const OBSERVED_AT_MS: u64 = 5_000;

#[derive(Clone, Copy)]
pub(super) enum HedgeCase {
    Eligible,
    BeforeTail,
    Linked,
    PrimaryUnavailable,
}

impl HedgeCase {
    pub(super) const fn primary_launched_at_ms(self) -> u64 {
        match self {
            Self::Eligible | Self::Linked | Self::PrimaryUnavailable => 1_000,
            Self::BeforeTail => 4_000,
        }
    }

    pub(super) const fn linked(self) -> bool {
        matches!(self, Self::Linked)
    }
}

pub(super) fn mirror_plan(case: HedgeCase) -> PlannedWork {
    let post = PostId::new("current");
    let mut state = state(post.clone());
    let active = active::actions(&state, post.clone(), case);
    let stats = stats::model(case);
    let evidence = evidence::PlanEvidence::new(post.clone());
    state.apply_playback(playback_for(post, 0));
    evidence.plan(&mut state, &stats, &active)
}

fn state(post: PostId) -> DeliveryState {
    let meta = VideoMeta {
        urls: vec![PRIMARY.into(), ALTERNATE.into()],
        delivery: DeliveryKind::Progressive,
        sha256: Some("11".repeat(32)),
        size_bytes: Some(1_000_000),
        duration_ms: Some(8_000),
    };
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_focus(
        DeliveryFocus::compatibility(
            vec![FocusItem {
                post: post.clone(),
                meta,
            }],
            0,
            0,
        ),
        0,
    );
    learn_sources(&mut state, &post);
    state
}

fn learn_sources(state: &mut DeliveryState, post: &PostId) {
    for source in [PRIMARY, ALTERNATE] {
        let identity = state.catalog().transfer_identity(post, source).unwrap();
        state.catalog_mut().learn_response_for(
            &identity,
            ghostr_engine::catalog::LearnedFacts {
                content_length: Some(1_000_000),
                accept_ranges: Some(true),
                host: None,
            },
        );
    }
}
