
use crate::delivery_events::{command_channel, PlayerPreparationAttempt, PlayerPreparationAuthority, PlayerPreparationIngress, PlayerPreparationObservation, PlayerPreparationReport, PlayerPreparationState};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn mailbox_preserves_an_exact_attempt_lifecycle_and_reset_fence() {
    let (handle, receiver) = command_channel();
    let ticket = handle.player_preparation_admission();
    assert_eq!(
        handle.report_player_preparation_initial(
            ticket,
            report(1, 1, 1, PlayerPreparationState::Initializing),
        ),
        PlayerPreparationIngress::Accepted,
    );
    assert_eq!(
        handle.report_player_preparation(report(
            1,
            1,
            2,
            PlayerPreparationState::FirstFrameRendered,
        )),
        PlayerPreparationIngress::Accepted,
    );
    assert_eq!(
        handle.report_player_preparation(report(
            1,
            1,
            3,
            PlayerPreparationState::Released,
        )),
        PlayerPreparationIngress::Accepted,
    );
    let states: Vec<_> = core::iter::from_fn(|| receiver.try_player_preparation())
        .map(|report| report.state())
        .collect();
    assert_eq!(
        states,
        [
            PlayerPreparationState::Initializing,
            PlayerPreparationState::FirstFrameRendered,
            PlayerPreparationState::Released,
        ]
    );
    assert_eq!(
        handle.report_player_preparation(report(
            1,
            1,
            4,
            PlayerPreparationState::Released,
        )),
        PlayerPreparationIngress::Stale,
    );
    let stale_ticket = handle.player_preparation_admission();
    receiver.discard_pending();
    assert_eq!(
        handle.report_player_preparation_initial(
            stale_ticket,
            report(2, 2, 1, PlayerPreparationState::Initializing),
        ),
        PlayerPreparationIngress::InvalidAdmission,
    );
}

fn report(
    post_index: u64,
    attempt_generation: u64,
    sequence: u64,
    state: PlayerPreparationState,
) -> PlayerPreparationReport {
    let post = PostId::new(format!("p{post_index}"));
    let binding = Catalog::new().upsert(post.clone(), meta(post_index));
    let authority = PlayerPreparationAuthority::try_new(
        post,
        binding,
        ContentRevision::default(),
        format!("asset-{post_index}"),
    )
    .expect("valid test fixture");
    let attempt = PlayerPreparationAttempt::try_new(1, 7, attempt_generation).expect("valid test fixture");
    let observation = PlayerPreparationObservation::try_new(state, None, sequence).expect("valid test fixture");
    PlayerPreparationReport::try_new(authority, attempt, sequence, observation).expect("valid test fixture")
}

fn meta(index: u64) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://media.example/p{index}.mp4")],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(2_000),
    }
}
