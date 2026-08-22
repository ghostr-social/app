use super::failure_class;
use ghostr_engine::origin_model::ErrorReason;

#[test]
fn every_typed_hls_failure_has_a_fixed_privacy_safe_terminal_class() {
    let expected = [
        (ErrorReason::Timeout, "warp_hls_timeout"),
        (ErrorReason::Dns, "warp_hls_dns"),
        (ErrorReason::Tls, "warp_hls_tls"),
        (ErrorReason::Http4xx, "warp_hls_http_4xx"),
        (ErrorReason::Http5xx, "warp_hls_http_5xx"),
        (ErrorReason::InvalidResponse, "warp_hls_invalid_response"),
        (
            ErrorReason::RangeNoncompliant,
            "warp_hls_range_noncompliant",
        ),
        (ErrorReason::Connection, "warp_hls_connection"),
        (ErrorReason::Policy, "warp_hls_policy"),
        (ErrorReason::Unknown, "warp_hls_unknown"),
    ];

    for (reason, class) in expected {
        assert_eq!(failure_class(reason), class);
    }
}
