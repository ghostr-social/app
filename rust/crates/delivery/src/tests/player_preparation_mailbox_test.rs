use crate::delivery_events::{
    command_channel, DeliveryHandle, PlayerPreparationAttempt, PlayerPreparationAuthority,
    PlayerPreparationIngress, PlayerPreparationObservation, PlayerPreparationReport,
    PlayerPreparationState,
};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::ContentRevision;
use PlayerPreparationIngress::{Accepted, Saturated, Stale};

#[test]
fn preparation_mailbox_is_bounded_and_keeps_each_posts_latest_evidence() {
    let (handle, mut receiver) = command_channel();
    for index in 0..4 {
        assert_eq!(send(&handle, index, stamp(1, 7, 11, 1)), Accepted);
    }
    assert_eq!(send(&handle, 4, stamp(1, 7, 11, 1)), Saturated);
    assert_eq!(send(&handle, 0, stamp(1, 7, 11, 3)), Accepted);
    assert_eq!(send(&handle, 0, stamp(1, 7, 11, 2)), Stale);
    assert_eq!(send(&handle, 0, stamp(1, 6, 99, 99)), Stale);
    assert_eq!(send(&handle, 0, stamp(2, 7, 11, 4)), Stale);
    assert_eq!(send(&handle, 0, stamp(2, 7, 12, 1)), Accepted);
    assert_eq!(send(&handle, 0, stamp(1, 7, 11, 99)), Stale);
    assert_eq!(send(&handle, 0, stamp(2, 8, 1, 1)), Accepted);

    let reports: Vec<_> = std::iter::from_fn(|| receiver.try_player_preparation()).collect();

    assert_eq!(reports.len(), 4);
    assert_eq!(
        reports
            .iter()
            .find(|item| item.post().as_str() == "p0")
            .unwrap()
            .client_epoch(),
        8
    );
    assert!(reports.iter().any(|item| item.post().as_str() == "p1"));
}

fn send(handle: &DeliveryHandle, index: u64, stamp: Stamp) -> PlayerPreparationIngress {
    handle.report_player_preparation(report(index, stamp))
}

struct Stamp {
    capability: u64,
    epoch: u64,
    attempt: u64,
    sequence: u64,
}

fn stamp(capability: u64, epoch: u64, attempt: u64, sequence: u64) -> Stamp {
    Stamp {
        capability,
        epoch,
        attempt,
        sequence,
    }
}

fn report(index: u64, stamp: Stamp) -> PlayerPreparationReport {
    let post = PostId::new(format!("p{index}"));
    let meta = VideoMeta {
        urls: vec![format!("https://media.example/p{index}.mp4")],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(2_000),
    };
    let binding = Catalog::new().upsert(post.clone(), meta);
    PlayerPreparationReport::try_new(
        PlayerPreparationAuthority::try_new(post, binding, ContentRevision::default()).unwrap(),
        PlayerPreparationAttempt::try_new(stamp.capability, stamp.epoch, stamp.attempt).unwrap(),
        stamp.sequence,
        PlayerPreparationObservation::try_new(
            PlayerPreparationState::Initialized,
            None,
            stamp.sequence,
        )
        .unwrap(),
    )
    .unwrap()
}
