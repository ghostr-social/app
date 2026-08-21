pub(super) fn is_admission_timeout(error: &anyhow::Error) -> bool {
    error.is::<ghostr_net::media_request_executor::MediaRequestAdmissionTimeout>()
}

pub(super) fn unix_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}
