
use crate::delivery_events::{DeliveryFocus, FocusItem, PlayerPreparationAttempt, PlayerPreparationAuthority, PlayerPreparationObservation, PlayerPreparationReport, PlayerPreparationState};
use crate::manager::state::DeliveryState;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::ContentRevision;

const POST: &str = "hashed";
const SOURCE: &str = "https://media.example/hashed.mp4";

#[test]
fn advertised_hash_is_not_persistent_before_local_verification() {
    let mut state = hashed_state();
    reject_decoder(&mut state);

    assert_eq!(persistent_records(&state), 0);
}

#[test]
fn locally_verified_hash_makes_decoder_evidence_persistent() {
    let mut state = hashed_state();
    let post = PostId::new(POST);
    let identity = state.catalog().transfer_identity(&post, SOURCE).expect("valid test fixture");
    assert!(state.catalog_mut().record_verified_hash_for(
        &identity,
        &"a".repeat(64),
        SOURCE,
        1,
    ));
    reject_decoder(&mut state);

    assert_eq!(persistent_records(&state), 1);
}

fn hashed_state() -> DeliveryState {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_focus(
        DeliveryFocus::compatibility(vec![FocusItem {
            post: PostId::new(POST),
            meta: VideoMeta {
                urls: vec![SOURCE.into()], delivery: DeliveryKind::Progressive,
                sha256: Some("a".repeat(64)), size_bytes: Some(16), duration_ms: Some(2_000),
            },
        }], 0, 0),
        1,
    );
    state
}

fn reject_decoder(state: &mut DeliveryState) {
    apply(state, 1, PlayerPreparationState::Initializing, None);
    apply(state, 2, PlayerPreparationState::Failed, Some("decoderUnsupported"));
}

fn apply(state: &mut DeliveryState, sequence: u64, phase: PlayerPreparationState, failure: Option<&str>) {
    let post = PostId::new(POST);
    let authority = PlayerPreparationAuthority::try_new(
        post.clone(), state.catalog().binding(&post).expect("valid test fixture"), ContentRevision::default(), "asset",
    ).expect("valid test fixture");
    let report = PlayerPreparationReport::try_new(
        authority, PlayerPreparationAttempt::try_new(7, 1, 1).expect("valid test fixture"), sequence,
        PlayerPreparationObservation::try_new(phase, failure.map(str::to_owned), sequence).expect("valid test fixture"),
    ).expect("valid test fixture");
    assert!(state.apply_player_preparation(report));
}

fn persistent_records(state: &DeliveryState) -> usize {
    serde_json::to_value(state.client_capabilities().state()).expect("valid test fixture")["records"]
        .as_array().expect("valid test fixture").len()
}
