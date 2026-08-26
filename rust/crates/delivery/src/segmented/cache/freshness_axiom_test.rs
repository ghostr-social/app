use super::*;

impl HlsCacheMetadata {
    pub(in crate::segmented) fn from_headers(headers: &HeaderMap) -> Self {
        Self::from_parts(headers, Duration::ZERO, true)
    }
}
