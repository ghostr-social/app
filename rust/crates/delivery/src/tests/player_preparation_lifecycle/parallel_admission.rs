use crate::delivery_events::{
    command_channel, PlayerPreparationAttempt, PlayerPreparationAuthority,
    PlayerPreparationIngress, PlayerPreparationObservation, PlayerPreparationReport,
    PlayerPreparationState,
};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn parallel_posts_accept_initials_that_begin_outside_allocation_order() {
    let (handle, _receiver) = command_channel();
    for (post, attempt) in [("p2", 4), ("p1", 3)] {
        let ticket = handle.player_preparation_admission();
        assert_eq!(
            handle.report_player_preparation_initial(ticket, report(post, attempt)),
            PlayerPreparationIngress::Accepted,
        );
    }
}

fn report(post: &str, attempt_generation: u64) -> PlayerPreparationReport {
    let post = PostId::new(post);
    let binding = Catalog::new().upsert(post.clone(), meta());
    let authority = PlayerPreparationAuthority::try_new(
        post,
        binding,
        ContentRevision::default(),
        "asset",
    )
    .unwrap();
    let attempt = PlayerPreparationAttempt::try_new(1, 7, attempt_generation).unwrap();
    let observation = PlayerPreparationObservation::try_new(
        PlayerPreparationState::Initializing,
        None,
        attempt_generation,
    )
    .unwrap();
    PlayerPreparationReport::try_new(authority, attempt, 1, observation).unwrap()
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
