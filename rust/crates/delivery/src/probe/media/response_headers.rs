use ghostr_engine::evidence::EvidenceValidator;
use ghostr_net::media_request_executor::MediaResponse;
use reqwest::header::{
    HeaderName, ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_TYPE, ETAG, LAST_MODIFIED,
};

pub(super) fn content_length(response: &MediaResponse) -> Option<u64> {
    text(response, &CONTENT_LENGTH)?.trim().parse().ok()
}

pub(super) fn content_type(response: &MediaResponse) -> Option<String> {
    text(response, &CONTENT_TYPE)
}

pub(super) fn accepts_byte_ranges(response: &MediaResponse) -> Option<bool> {
    text(response, &ACCEPT_RANGES).map(|value| value.trim().eq_ignore_ascii_case("bytes"))
}

pub(super) fn validator(response: &MediaResponse) -> Option<EvidenceValidator> {
    text(response, &ETAG)
        .and_then(EvidenceValidator::strong_etag)
        .or_else(|| text(response, &LAST_MODIFIED).and_then(EvidenceValidator::last_modified))
}

fn text(response: &MediaResponse, name: &HeaderName) -> Option<String> {
    response
        .headers()
        .get(name)?
        .to_str()
        .ok()
        .map(str::to_owned)
}
