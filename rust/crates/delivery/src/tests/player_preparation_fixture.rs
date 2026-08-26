
use crate::delivery_events::{DeliveryFocus, FocusItem, PlayerPreparationAttempt, PlayerPreparationAuthority, PlayerPreparationObservation, PlayerPreparationReport, PlayerPreparationState};
use crate::manager::state::DeliveryState;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::ContentRevision;

#[derive(Clone, Copy)]
pub(super) struct EvidenceSpec<'a> {
    pub(super) post: &'a str,
    pub(super) revision: ContentRevision,
    pub(super) sequence: u64,
    pub(super) state: PlayerPreparationState,
}

pub(super) fn state(ids: &[&str], current_index: usize) -> DeliveryState {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_focus(focus(ids, current_index), 1);
    state
}

pub(super) fn focus(ids: &[&str], current_index: usize) -> DeliveryFocus {
    DeliveryFocus::compatibility(
        ids.iter()
            .map(|id| FocusItem {
                post: PostId::new(*id),
                meta: meta(id),
            })
            .collect(),
        current_index,
        0,
    )
}

pub(super) fn evidence(state: &DeliveryState, spec: EvidenceSpec<'_>) -> PlayerPreparationReport {
    let post = PostId::new(spec.post);
    let failure = (spec.state == PlayerPreparationState::Failed).then(|| "decode".to_owned());
    PlayerPreparationReport::try_new(
        PlayerPreparationAuthority::try_new(
            post.clone(),
            state.catalog().binding(&post).expect("valid test fixture"),
            spec.revision,
            format!("asset-{}", spec.post),
        )
        .expect("valid test fixture"),
        PlayerPreparationAttempt::try_new(1, 9, 4).expect("valid test fixture"),
        spec.sequence,
        PlayerPreparationObservation::try_new(spec.state, failure, spec.sequence).expect("valid test fixture"),
    )
    .expect("valid test fixture")
}

pub(super) fn report(
    post: &str,
    attempt_generation: u64,
    observed_us: u64,
) -> PlayerPreparationReport {
    report_with_epoch(post, 7, attempt_generation, observed_us)
}

pub(super) fn report_with_epoch(
    post: &str,
    client_epoch: u64,
    attempt_generation: u64,
    observed_us: u64,
) -> PlayerPreparationReport {
    let post = PostId::new(post);
    let binding = ghostr_engine::catalog::Catalog::new().upsert(post.clone(), meta(post.as_str()));
    let authority = PlayerPreparationAuthority::try_new(
        post,
        binding,
        ContentRevision::default(),
        "asset",
    )
    .expect("valid test fixture");
    let attempt = PlayerPreparationAttempt::try_new(1, client_epoch, attempt_generation).expect("valid test fixture");
    let observation = PlayerPreparationObservation::try_new(
        PlayerPreparationState::Initializing,
        None,
        observed_us,
    )
    .expect("valid test fixture");
    PlayerPreparationReport::try_new(authority, attempt, 1, observation).expect("valid test fixture")
}

pub(super) fn meta(id: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://media.example/{id}.mp4")],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(2_000),
    }
}
