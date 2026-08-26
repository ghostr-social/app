use ghostr_engine::representation::{HttpGenerationAuthority, HttpGenerationKey};

#[derive(Clone)]
pub(in crate::partial_range_store) struct HttpGenerationState {
    pub(super) source: String,
    pub(super) key: Option<HttpGenerationKey>,
    pub(super) authority: Option<HttpGenerationAuthority>,
}
