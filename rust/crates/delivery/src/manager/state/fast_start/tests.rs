mod fixture;

use super::FastStartEvidence;
use crate::delivery_events::{
    PlayerPreparationAttempt, PlayerPreparationAuthority, PlayerPreparationObservation,
    PlayerPreparationReport, PlayerPreparationState,
};
use fixture::{binding, store, tail_indexed_mp4};
use ghostr_engine::media_timeline::{parse_mp4_segments, MediaSegment};
use ghostr_partial_store::partial_range_store::ContentRevision;

#[tokio::test]
async fn evidence_requires_finalized_exact_bytes_and_current_player_authority() {
    let binding = binding();
    let bytes = tail_indexed_mp4();
    let timeline = parse_mp4_segments(&[MediaSegment::new(0, &bytes)]).unwrap();
    let (root, store) = store("fast-start-authority");
    store.bind_representation(binding.clone()).await.unwrap();
    store
        .set_total_len("next", bytes.len() as u64)
        .await
        .unwrap();
    store.write_range("next", 0, &bytes[..16]).await.unwrap();
    let incomplete = store.media_snapshot("next").await.unwrap();
    assert!(FastStartEvidence::from_snapshot(&binding, &incomplete, &timeline).is_none());
    store.write_range("next", 16, &bytes[16..]).await.unwrap();
    let unfinalized = store.media_snapshot("next").await.unwrap();
    assert!(FastStartEvidence::from_snapshot(&binding, &unfinalized, &timeline).is_none());
    store.finalize("next", None).await.unwrap();
    let finalized = store.media_snapshot("next").await.unwrap();
    let evidence = FastStartEvidence::from_snapshot(&binding, &finalized, &timeline).unwrap();

    assert!(evidence.matches(&report(&binding, finalized.revision(), 2), Some(2)));
    assert!(!evidence.matches(&report(&binding, ContentRevision::default(), 2), Some(2)));
    assert!(!evidence.matches(&report(&binding, finalized.revision(), 1), Some(2)));
    std::fs::remove_dir_all(root).ok();
}

fn report(
    binding: &ghostr_engine::representation::RepresentationBinding,
    revision: ContentRevision,
    generation: u64,
) -> PlayerPreparationReport {
    let authority =
        PlayerPreparationAuthority::try_new(binding.post().clone(), binding.clone(), revision)
            .unwrap();
    let observation = PlayerPreparationObservation::try_new(
        PlayerPreparationState::Failed,
        Some("invalidVideoTrack".into()),
        2,
    )
    .unwrap();
    PlayerPreparationReport::try_new(
        authority,
        PlayerPreparationAttempt::try_new(generation, 1, 1).unwrap(),
        2,
        observation,
    )
    .unwrap()
}
