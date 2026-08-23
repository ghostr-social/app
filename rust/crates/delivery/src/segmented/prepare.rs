use super::fetch::FetchedObject;
use std::sync::Arc;
use url::Url;

mod completion;
pub(in crate::segmented) use completion::{prepare_complete, PreparedComplete};

#[derive(Clone)]
pub(super) struct PreparedObject {
    pub request_url: String,
    pub final_url: Url,
    pub body: Arc<[u8]>,
    pub content_type: Option<String>,
    pub cache: super::cache::HlsCacheMetadata,
}

impl From<FetchedObject> for PreparedObject {
    fn from(object: FetchedObject) -> Self {
        Self {
            request_url: object.request_url,
            final_url: object.final_url,
            body: object.body,
            content_type: object.content_type,
            cache: object.cache,
        }
    }
}
