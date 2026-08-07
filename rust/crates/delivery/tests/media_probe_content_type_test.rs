mod range_fixture;

use ghostr_engine::host_stats::HostStats;
use ghostr_delivery::media_probe::probe;
use ghostr_net::transfer_timeouts::TransferTimeouts;

#[tokio::test]
async fn media_probe_rejects_an_explicit_image_content_type() {
    let url = range_fixture::content_type::serve(Some("image/jpeg"), range_fixture::body()).await;
    let client = range_fixture::media_client();
    let mut stats = HostStats::new();

    let result = probe(&client, &url, TransferTimeouts::default(), &mut stats).await;

    assert!(result.is_err(), "an image origin must not probe as video");
}

#[tokio::test]
async fn media_probe_allows_video_hls_generic_and_absent_content_types() {
    let allowed = [
        Some("video/mp4; charset=binary"),
        Some("application/vnd.apple.mpegurl"),
        Some("application/x-mpegurl"),
        Some("application/octet-stream"),
        None,
    ];
    let client = range_fixture::media_client();
    for content_type in allowed {
        let url = range_fixture::content_type::serve(content_type, range_fixture::body()).await;
        let mut stats = HostStats::new();
        let result = probe(&client, &url, TransferTimeouts::default(), &mut stats).await;
        assert!(result.is_ok(), "{content_type:?} should be admissible");
    }
}
