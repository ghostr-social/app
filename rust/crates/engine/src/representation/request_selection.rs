use super::SourceGeneration;
use serde::{Deserialize, Serialize};

/// Digest of the request fields that select an HTTP response variant.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct RequestSelection([u8; 32]);

impl RequestSelection {
    pub const fn new(digest: [u8; 32]) -> Self {
        Self(digest)
    }
}

impl SourceGeneration {
    pub const fn request_selection(&self) -> Option<RequestSelection> {
        self.request_selection
    }

    pub fn with_request_selection(mut self, selection: Option<RequestSelection>) -> Self {
        self.request_selection = selection;
        self
    }
}
