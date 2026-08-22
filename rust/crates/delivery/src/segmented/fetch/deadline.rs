use tokio::time::Instant;

pub(super) fn header_context(header_deadline: Instant, total_deadline: Instant) -> &'static str {
    if header_deadline == total_deadline {
        "HLS object transfer timed out"
    } else {
        "HLS response headers timed out"
    }
}
