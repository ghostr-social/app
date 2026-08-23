use crate::delivery_events::{
    command_channel, PlayerPreparationAttempt, PlayerPreparationAuthority,
    PlayerPreparationIngress, PlayerPreparationObservation, PlayerPreparationReport,
    PlayerPreparationState,
};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn admitted_terminal_keeps_reserved_capacity_under_nonterminal_pressure() {
    let (handle, mut receiver) = command_channel();
    let ticket = handle.player_preparation_admission();
    for post in 0..16 {
        assert_eq!(
            handle.report_player_preparation_initial(
                ticket,
                report(post, 1, PlayerPreparationState::Initializing),
            ),
            PlayerPreparationIngress::Accepted,
        );
    }
    while receiver.try_player_preparation().is_some() {}
    for post in 0..16 {
        let _ = handle.report_player_preparation(report(
            post,
            2,
            PlayerPreparationState::Initialized,
        ));
        let _ = handle.report_player_preparation(report(
            post,
            3,
            PlayerPreparationState::FirstFrameRendered,
        ));
    }

    assert_eq!(
        handle.report_player_preparation(report(0, 4, PlayerPreparationState::Released)),
        PlayerPreparationIngress::Accepted,
    );
    assert!(std::iter::from_fn(|| receiver.try_player_preparation())
        .any(|report| report.post() == &PostId::new("p0") && report.is_terminal()));
}

fn report(post: u64, sequence: u64, state: PlayerPreparationState) -> PlayerPreparationReport {
    let attempt_generation = post + 1;
    let post = PostId::new(format!("p{post}"));
    let binding = Catalog::new().upsert(post.clone(), meta());
    let authority = PlayerPreparationAuthority::try_new(
        post,
        binding,
        ContentRevision::default(),
        "asset",
    )
    .unwrap();
    let attempt = PlayerPreparationAttempt::try_new(1, 7, attempt_generation).unwrap();
    let observation = PlayerPreparationObservation::try_new(state, None, sequence).unwrap();
    PlayerPreparationReport::try_new(authority, attempt, sequence, observation).unwrap()
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://media.example/video.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(2_000),
    }
}
