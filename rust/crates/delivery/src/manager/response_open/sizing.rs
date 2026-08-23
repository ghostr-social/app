use super::ResponseObservation;

pub(super) fn response_bytes(response: ResponseObservation) -> u64 {
    match response {
        ResponseObservation::Rejected(_) | ResponseObservation::Ignored { .. } => 0,
        ResponseObservation::Partial { range, .. } => range.len(),
        ResponseObservation::Body { request, .. } => request.reserved_network_bytes(),
    }
}
