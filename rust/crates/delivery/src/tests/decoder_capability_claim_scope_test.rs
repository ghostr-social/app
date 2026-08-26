
use crate::delivery_events::{DeliveryFocus, FocusItem, PlayerPreparationAttempt, PlayerPreparationAuthority, PlayerPreparationObservation, PlayerPreparationReport, PlayerPreparationState};
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::PlannerCapability;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn unverified_hash_claim_cannot_block_another_posts_source() {
    let mut state = claimed_state();
    apply(&mut state, "first", 1, PlayerPreparationState::Initializing);
    apply(
        &mut state,
        "first",
        2,
        PlayerPreparationState::Failed,
    );

    assert_eq!(
        state.planner_capability(&PostId::new("second"), 2),
        PlannerCapability::Unavailable,
    );
}

fn claimed_state() -> DeliveryState {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    let items = ["first", "second"].map(|id| FocusItem {
        post: PostId::new(id),
        meta: VideoMeta {
            urls: vec![format!("https://media.example/{id}.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: Some("a".repeat(64)),
            size_bytes: Some(16),
            duration_ms: Some(2_000),
        },
    });
    state.apply_focus(DeliveryFocus::compatibility(items.into(), 0, 0), 1);
    state
}

fn apply(
    state: &mut DeliveryState,
    id: &str,
    sequence: u64,
    phase: PlayerPreparationState,
) {
    let post = PostId::new(id);
    let authority = PlayerPreparationAuthority::try_new(
        post,
        state.catalog().binding(&PostId::new(id)).expect("valid test fixture"),
        ContentRevision::default(),
        format!("asset-{id}"),
    )
    .expect("valid test fixture");
    let report = PlayerPreparationReport::try_new(
        authority,
        PlayerPreparationAttempt::try_new(7, 1, 1).expect("valid test fixture"),
        sequence,
        PlayerPreparationObservation::try_new(
            phase,
            matches!(phase, PlayerPreparationState::Failed)
                .then(|| "decoderUnsupported".to_owned()),
            sequence,
        )
        .expect("valid test fixture"),
    )
    .expect("valid test fixture");
    assert!(state.apply_player_preparation(report));
}
