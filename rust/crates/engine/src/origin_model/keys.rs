use super::{OriginContext, RequestMethod};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub(super) struct OriginContextKey {
    pub origin: String,
    pub context: OriginContext,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub(super) struct UrlContextKey {
    pub url_id: String,
    pub context: OriginContext,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub(super) struct OriginMethodKey {
    pub origin: String,
    pub method: RequestMethod,
}
