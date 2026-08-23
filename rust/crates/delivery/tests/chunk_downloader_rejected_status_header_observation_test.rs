mod range_fixture;

use ghostr_delivery::chunk::downloader::{ResponseObservation, ResponseRejection};

const REJECTED: &[u8] = b"HTTP/1.1 503 Service Unavailable\r\n\
Content-Type: video/mp4\r\n\
ETag: \"down-v1\"\r\n\
Content-Length: 0\r\n\
Connection: close\r\n\r\n";

#[tokio::test]
async fn rejected_status_still_reports_bounded_final_headers() {
    let (result, observed, opened) =
        range_fixture::header_failure::download(REJECTED, "rejected-header-evidence").await;

    assert!(result.is_err());
    assert_eq!(opened, 1, "TTFB must survive status rejection");
    assert_eq!(observed.len(), 1);
    assert_eq!(
        observed[0].observation(),
        ResponseObservation::Rejected(ResponseRejection::Status)
    );
    assert_eq!(observed[0].evidence().status, 503);
    assert_eq!(observed[0].evidence().validator, None);
    assert_eq!(observed[0].evidence().content_type, None);
}
