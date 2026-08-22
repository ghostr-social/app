use ghostr_engine::origin_model::ErrorReason;

pub(super) const fn class(reason: ErrorReason) -> &'static str {
    match reason {
        ErrorReason::Timeout => "warp_hls_timeout",
        ErrorReason::Dns => "warp_hls_dns",
        ErrorReason::Tls => "warp_hls_tls",
        ErrorReason::Http4xx => "warp_hls_http_4xx",
        ErrorReason::Http5xx => "warp_hls_http_5xx",
        ErrorReason::InvalidResponse => "warp_hls_invalid_response",
        ErrorReason::RangeNoncompliant => "warp_hls_range_noncompliant",
        ErrorReason::Connection => "warp_hls_connection",
        ErrorReason::Policy => "warp_hls_policy",
        ErrorReason::Unknown => "warp_hls_unknown",
    }
}
