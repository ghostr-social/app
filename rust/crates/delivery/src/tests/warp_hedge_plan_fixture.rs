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
    AdvertisedOnly,
    PrimaryVerifiedOnly,
    AlternateRotated,
    BeforeTail,
    Linked,
    PrimaryUnavailable,
}

impl HedgeCase {
    const fn primary_launched_at_ms(self) -> u64 {
        match self {
            Self::BeforeTail => 4_000,
            _ => 1_000,
        }
    }

    const fn linked(self) -> bool {
        matches!(self, Self::Linked)
    }
}

pub(super) fn mirror_plan(case: HedgeCase) -> PlannedWork {
    mirror_plan_on_network(case, ghostr_engine::origin_model::NetworkClass::Unavailable)
}

pub(super) fn mirror_plan_on_network(
    case: HedgeCase,
    network_class: ghostr_engine::origin_model::NetworkClass,
) -> PlannedWork {
    let post = PostId::new("current");
    let mut state = state(&post, network_class, case);
    let active = active::actions(&state, post.clone(), case);
    let stats = stats::model(case);
    let evidence = evidence::PlanEvidence::new(post.clone());
    state.apply_playback(&playback_for(post, 0));
    evidence.plan(&state, &stats, &active)
}

fn state(
    post: &PostId,
    network_class: ghostr_engine::origin_model::NetworkClass,
    case: HedgeCase,
) -> DeliveryState {
    let meta = VideoMeta {
        urls: vec![PRIMARY.into(), ALTERNATE.into()],
        delivery: DeliveryKind::Progressive,
        sha256: Some("11".repeat(32)),
        size_bytes: Some(1_000_000),
        duration_ms: Some(8_000),
    };
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_network_status(crate::delivery_events::DeliveryNetworkStatus::new(
        network_class,
        1,
    ));
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
    evidence::learn_identity(&mut state, post, case);
    state
}
