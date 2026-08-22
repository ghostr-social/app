mod probe_fixture;
mod range_fixture;

use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::host_stats::HostStats;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use probe_fixture::probe;

#[tokio::test]
async fn media_probe_head_learns_length_range_support_and_content_type() {
    let url = range_fixture::ranged::serve_ranged(range_fixture::body()).await;
    let client = range_fixture::media_client();
    let mut stats = HostStats::new();

    let result = probe(&client, &url, TransferTimeouts::default(), &mut stats)
        .await
        .expect("probe");

    assert_eq!(result.content_length, Some(16));
    assert_eq!(result.accept_ranges, Some(true));
    assert_eq!(result.content_type.as_deref(), Some("video/mp4"));
    assert_eq!(
        result.validator,
        EvidenceValidator::strong_etag("\"fixture-ranged\"")
    );
}
