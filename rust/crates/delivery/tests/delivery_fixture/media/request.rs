use super::Recorder;
use axum::http::{header, HeaderMap, Method};

/// Logs one attempt and reports its requested byte range.
pub(super) fn note(state: &Recorder, method: &Method, headers: &HeaderMap) -> Option<(u64, u64)> {
    let range = super::response::requested(headers, state.bytes.len() as u64);
    let request = match range {
        Some((start, end)) => format!("{}:{method}:{start}-{end}", state.tag),
        None => format!("{}:{method}:full", state.tag),
    };
    let validator = headers
        .get(header::IF_RANGE)
        .and_then(|value| value.to_str().ok())
        .map(|value| format!(":if-range={value}"))
        .unwrap_or_default();
    state
        .log
        .lock()
        .expect("hit log")
        .push(format!("{request}{validator}"));
    range
}
