mod range_fixture;

use ghostr_delivery::manager::failure::{classify, FailureClass};
use ghostr_delivery::probe::media::probe;
use ghostr_engine::host_stats::HostStats;
use ghostr_net::transfer_timeouts::TransferTimeouts;

#[tokio::test]
async fn unsupported_origin_media_type_is_diagnostic_and_permanent() {
    let url = range_fixture::content_type::serve(
        Some("image/jpeg; charset=binary"),
        range_fixture::body(),
    )
    .await;
    let mut stats = HostStats::new();
    let error = probe(
        &range_fixture::media_client(),
        &url,
        TransferTimeouts::default(),
        &mut stats,
    )
    .await
    .expect_err("image origin must be rejected");

    assert_eq!(classify(&error), FailureClass::Permanent);
    assert!(error
        .to_string()
        .contains("unsupported Content-Type \"image/jpeg\""));
}
