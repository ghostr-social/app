#[path = "chunk_downloader_admission_observation_test/mod.rs"]
mod observation_fixture;
mod range_fixture;
use core::time::Duration;

#[tokio::test]
async fn local_admission_wait_is_not_trained_as_origin_time_or_concurrency() {
    let observation = Box::pin(observation_fixture::observe()).await;

    assert_eq!(observation.concurrency, 2);
    assert_eq!(observation.throughput_samples, 1);
    assert_eq!(observation.bytes_written, observation_fixture::BODY_BYTES);
    assert!(
        observation.excluded >= Duration::from_millis(450),
        "only {:?} was excluded from origin training",
        observation.excluded
    );
}
