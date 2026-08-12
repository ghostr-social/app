use ghostr_engine::host_stats::host_of;

pub(crate) fn origin_key(url: &str) -> String {
    host_of(url).unwrap_or_else(|| url.to_owned())
}
