use super::Catalog;
use crate::representation::TransferIdentity;
use crate::{PostId, VideoMeta};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

pub(super) const MAX_REJECTIONS: usize = 4_096;

/// Persistent negative evidence contains no access URL or signed query token.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub(super) struct SourceRejection {
    post: String,
    expected_digest: String,
    endpoint: [u8; 32],
}

impl SourceRejection {
    fn new(post: &PostId, digest: &str, source: &str) -> Self {
        Self {
            post: post.as_str().to_owned(),
            expected_digest: digest.to_ascii_lowercase(),
            endpoint: Sha256::digest(source.as_bytes()).into(),
        }
    }
}

impl Catalog {
    pub(super) fn reject_source(&mut self, identity: &TransferIdentity, digest: &str) {
        let rejection = SourceRejection::new(identity.post(), digest, identity.source().as_str());
        if self.quarantined_sources.contains(&rejection) {
            return;
        }
        if self.quarantined_sources.len() >= MAX_REJECTIONS {
            self.quarantined_sources.pop_first();
        }
        self.quarantined_sources.insert(rejection);
        self.reliability_revision = self.reliability_revision.saturating_add(1);
    }

    pub(super) fn source_rejected(&self, post: &PostId, meta: &VideoMeta, source: &str) -> bool {
        meta.sha256.as_deref().is_some_and(|digest| {
            self.quarantined_sources
                .contains(&SourceRejection::new(post, digest, source))
        })
    }

    pub(super) fn quarantined(&self, post: &PostId, meta: &VideoMeta) -> bool {
        !meta.urls.is_empty()
            && meta
                .urls
                .iter()
                .all(|source| self.source_rejected(post, meta, source))
    }
}
