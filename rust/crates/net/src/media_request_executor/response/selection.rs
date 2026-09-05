use crate::media_retention::MediaRetention;
use ghostr_engine::representation::RequestSelection;
use reqwest::header::{
    HeaderMap, HeaderName, ACCEPT_ENCODING, AUTHORIZATION, COOKIE, PROXY_AUTHORIZATION, VARY,
};
use reqwest::Url;
use sha2::{Digest as _, Sha256};
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
pub(in crate::media_request_executor) enum ResponseSelection {
    Unvaried,
    Varied(RequestSelection),
    Transient,
}

impl ResponseSelection {
    pub(in crate::media_request_executor) fn capture(
        request: &HeaderMap,
        response: &HeaderMap,
    ) -> Self {
        if [AUTHORIZATION, COOKIE, PROXY_AUTHORIZATION]
            .iter()
            .any(|name| request.contains_key(name))
        {
            return Self::Transient;
        }
        if !response.contains_key(VARY) {
            return Self::Unvaried;
        }
        fields(response).map_or(Self::Transient, |fields| {
            Self::Varied(selection(request, fields))
        })
    }

    pub(super) const fn identity(self) -> Option<RequestSelection> {
        match self {
            Self::Varied(selection) => Some(selection),
            _ => None,
        }
    }

    pub(super) fn retention(self, headers: &HeaderMap, url: &Url) -> MediaRetention {
        match self {
            Self::Transient => MediaRetention::Transient,
            Self::Unvaried => MediaRetention::from_headers(headers, url),
            Self::Varied(_) => varied_retention(headers, url),
        }
    }
}

fn fields(response: &HeaderMap) -> Option<BTreeSet<String>> {
    let mut fields = BTreeSet::new();
    for value in response.get_all(VARY) {
        for field in value.to_str().ok()?.split(',') {
            let field = field.trim();
            if field == "*" || fields.len() >= 64 {
                return None;
            }
            fields.insert(
                HeaderName::from_bytes(field.as_bytes())
                    .ok()?
                    .as_str()
                    .to_owned(),
            );
        }
    }
    Some(fields)
}

fn selection(request: &HeaderMap, mut fields: BTreeSet<String>) -> RequestSelection {
    fields.insert(ACCEPT_ENCODING.as_str().to_owned());
    let mut digest = Sha256::new();
    digest.update(b"ghostr-request-selection-v1\0anonymous\0");
    for field in fields {
        hash_part(&mut digest, field.as_bytes());
        let values = request.get_all(field.as_str());
        digest.update((values.iter().count() as u64).to_be_bytes());
        for value in values {
            hash_part(&mut digest, value.as_bytes());
        }
    }
    RequestSelection::new(digest.finalize().into())
}

fn hash_part(digest: &mut Sha256, bytes: &[u8]) {
    digest.update((bytes.len() as u64).to_be_bytes());
    digest.update(bytes);
}

fn varied_retention(headers: &HeaderMap, url: &Url) -> MediaRetention {
    let mut headers = headers.clone();
    headers.remove(VARY);
    match MediaRetention::from_headers(&headers, url) {
        MediaRetention::Transient => MediaRetention::Transient,
        _ => MediaRetention::Partitioned,
    }
}
