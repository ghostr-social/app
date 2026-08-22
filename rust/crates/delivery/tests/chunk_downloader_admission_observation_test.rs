#[path = "chunk_downloader_admission_observation_test/mod.rs"]
mod observation_fixture;
mod range_fixture;

#[tokio::test]
async fn local_admission_wait_is_not_trained_as_origin_time_or_concurrency() {
    let observation = observation_fixture::observe().await;

    assert_eq!(observation.concurrency, 2);
    assert!(
        observation.throughput > 100_000.0,
        "observed throughput {}",
        observation.throughput
    );
}
