use super::asset_fixture::{exchange, range_values};
use axum::body::to_bytes;
use axum::http::StatusCode;

const RANGED_OK: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\nfull";
const UNKNOWN_TOTAL: &[u8] = b"HTTP/1.1 206 Partial Content\r\nContent-Length: 4\r\n\
Content-Range: bytes 4-7/*\r\nConnection: close\r\n\r\ngood";
const DUPLICATE_GEOMETRY: &[u8] = b"HTTP/1.1 206 Partial Content\r\nContent-Length: 4\r\n\
Content-Range: bytes 4-7/16\r\nContent-Range: bytes 8-11/16\r\n\
Connection: close\r\n\r\ngood";
const KNOWN_SUFFIX_PREFIX: &[u8] = b"HTTP/1.1 206 Partial Content\r\nContent-Length: 4\r\n\
Content-Range: bytes 8-11/16\r\nConnection: close\r\n\r\npart";
const UNSATISFIABLE: &[u8] = b"HTTP/1.1 416 Range Not Satisfiable\r\n\
Content-Range: bytes */16\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";

#[tokio::test]
async fn uncached_asset_status_and_total_must_match_the_request() {
    let ranged_ok = exchange(RANGED_OK.to_vec(), &["bytes=4-7"]).await;
    assert_eq!(
        ranged_ok.result.expect_err("ranged 200"),
        StatusCode::BAD_GATEWAY
    );

    let duplicate = exchange(DUPLICATE_GEOMETRY.to_vec(), &["bytes=4-7"]).await;
    assert_eq!(
        duplicate.result.expect_err("duplicate range"),
        StatusCode::BAD_GATEWAY
    );

    let suffix = exchange(UNKNOWN_TOTAL.to_vec(), &["bytes=-4"]).await;
    assert_eq!(
        suffix.result.expect_err("unknown suffix total"),
        StatusCode::BAD_GATEWAY
    );

    let bounded = exchange(UNKNOWN_TOTAL.to_vec(), &["bytes=4-9"]).await;
    let response = bounded.result.expect("bounded unknown total");
    assert_eq!(
        to_bytes(response.into_body(), 4)
            .await
            .expect("valid test fixture"),
        "good"
    );

    let open = exchange(UNKNOWN_TOTAL.to_vec(), &["bytes=4-"]).await;
    assert_eq!(
        open.result.expect("open unknown total").status(),
        StatusCode::PARTIAL_CONTENT
    );

    let known_suffix = exchange(KNOWN_SUFFIX_PREFIX.to_vec(), &["bytes=-8"]).await;
    assert_eq!(
        known_suffix.result.expect("known suffix").status(),
        StatusCode::PARTIAL_CONTENT
    );

    let large = exchange(UNKNOWN_TOTAL.to_vec(), &["bytes=4-99999999"]).await;
    assert_eq!(range_values(&large.requests[1]), ["bytes=4-99999999"]);
    assert_eq!(
        large.result.expect("large requested range").status(),
        StatusCode::PARTIAL_CONTENT
    );

    let lying = exchange(UNSATISFIABLE.to_vec(), &["bytes=4-7"]).await;
    assert_eq!(
        lying.result.expect_err("lying 416"),
        StatusCode::BAD_GATEWAY
    );

    let truthful = exchange(UNSATISFIABLE.to_vec(), &["bytes=20-23"]).await;
    assert_eq!(
        truthful.result.expect("truthful 416").status(),
        StatusCode::RANGE_NOT_SATISFIABLE
    );
}
