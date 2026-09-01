use super::super::CHUNK_BYTES;
use crate::delivery_fixture::full_recovery_origin::ObservedRequest;
use axum::http::Method;

pub(super) fn trial_slice_bytes(request: &ObservedRequest) -> Option<usize> {
    let valid = request.method == Method::GET
        && request.path == "/trial.mp4"
        && request.range.as_deref() == Some("bytes=0-4095")
        && request.encoding.as_deref() == Some("identity");
    valid.then_some(CHUNK_BYTES)?.try_into().ok()
}
