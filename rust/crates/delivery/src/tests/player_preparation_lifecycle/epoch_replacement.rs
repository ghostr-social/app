
use crate::delivery_events::{command_channel, PlayerPreparationAttempt, PlayerPreparationAuthority, PlayerPreparationIngress, PlayerPreparationObservation, PlayerPreparationReport, PlayerPreparationState};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn retired_client_epoch_cannot_replace_or_reenter_any_post() {
    let (handle, receiver) = command_channel();
    let ticket = handle.player_preparation_admission();
    assert_eq!(
        handle.report_player_preparation_initial(
            ticket,
            report("p1", 10, 1, PlayerPreparationState::Initializing),
        ),
        PlayerPreparationIngress::Accepted,
    );
    let older_ticket = handle.player_preparation_admission();
    assert_eq!(
        handle.report_player_preparation_initial(
            older_ticket,
            report("p1", 9, 1, PlayerPreparationState::Initializing),
        ),
        PlayerPreparationIngress::Stale,
    );
    assert_eq!(
        handle.report_player_preparation(report(
            "p1",
            10,
            2,
            PlayerPreparationState::FirstFrameRendered,
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
        ]
    );
    assert_eq!(
        handle.report_player_preparation(report(
            "p1",
            10,
            3,
            PlayerPreparationState::Released,
        )),
        PlayerPreparationIngress::Accepted,
    );
    for post in ["p1", "p2"] {
        let ticket = handle.player_preparation_admission();
        assert_eq!(
            handle.report_player_preparation_initial(
                ticket,
                report(post, 9, 1, PlayerPreparationState::Initializing),
            ),
            PlayerPreparationIngress::Stale,
        );
    }
}

fn report(
    post: &str,
    client_epoch: u64,
    sequence: u64,
    state: PlayerPreparationState,
) -> PlayerPreparationReport {
    let post = PostId::new(post);
    let binding = Catalog::new().upsert(post.clone(), meta());
    let authority = PlayerPreparationAuthority::try_new(
        post,
        binding,
        ContentRevision::default(),
        "asset-1",
    )
    .expect("valid test fixture");
    let attempt = PlayerPreparationAttempt::try_new(1, client_epoch, 1).expect("valid test fixture");
    let observation = PlayerPreparationObservation::try_new(state, None, sequence).expect("valid test fixture");
    PlayerPreparationReport::try_new(authority, attempt, sequence, observation).expect("valid test fixture")
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://media.example/p1.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(2_000),
    }
}
