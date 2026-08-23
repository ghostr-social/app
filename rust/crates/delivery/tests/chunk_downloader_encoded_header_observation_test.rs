mod range_fixture;

use ghostr_delivery::chunk::downloader::{ResponseObservation, ResponseRejection};

const ENCODED: &[u8] = b"HTTP/1.1 200 OK\r\n\
Content-Type: video/mp4\r\n\
Content-Encoding: gzip\r\n\
ETag: \"encoded-v1\"\r\n\
Content-Length: 0\r\n\
Connection: close\r\n\r\n";

#[tokio::test]
async fn unusable_content_encoding_still_reports_bounded_final_headers() {
    let (result, observed, opened) =
        range_fixture::header_failure::download(ENCODED, "encoded-header-evidence").await;

    assert!(result.is_err());
    assert_eq!(opened, 1, "TTFB must survive encoding rejection");
    assert_eq!(observed.len(), 1);
    assert_eq!(
        observed[0].observation(),
        ResponseObservation::Rejected(ResponseRejection::ContentEncoding)
    );
    assert_eq!(observed[0].evidence().status, 200);
    assert_eq!(observed[0].evidence().validator, None);
    assert_eq!(observed[0].evidence().content_type, None);
}
