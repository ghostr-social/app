use crate::adaptive::{ActionKind, ActionNode, RetrievalRequest};
use crate::PostId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct SoftRequestCommitment {
    post: PostId,
    source: String,
    request: RetrievalRequest,
}

impl SoftRequestCommitment {
    pub fn new(post: PostId, source: String, request: RetrievalRequest) -> Self {
        Self {
            post,
            source,
            request,
        }
    }

    fn admits(&self, action: &ActionNode) -> bool {
        self.post == action.post
            && self.source == action.origin
            && action.request() == Some(self.request)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(super) struct RequestScope {
    ordinary_tokens: u16,
    #[serde(default, skip_serializing_if = "is_zero")]
    hls_tokens: u16,
    soft: Vec<SoftRequestCommitment>,
}

impl RequestScope {
    pub(super) fn new(
        ordinary_tokens: u16,
        hls_tokens: u16,
        soft: Vec<SoftRequestCommitment>,
    ) -> Self {
        Self {
            ordinary_tokens,
            hls_tokens,
            soft,
        }
    }

    pub(super) fn admits(&self, action: &ActionNode, occupied: usize) -> bool {
        occupied < usize::from(self.ordinary_tokens)
            || (matches!(&action.kind, ActionKind::HlsBootstrap { .. })
                && occupied < usize::from(self.hls_tokens))
            || self.soft.iter().any(|item| item.admits(action))
    }

    pub(super) fn replay_sources(&self) -> Vec<String> {
        self.soft.iter().map(|item| item.source.clone()).collect()
    }

    pub(super) fn replay_project(
        &self,
        post: &impl Fn(&str) -> String,
        source: &impl Fn(&str) -> String,
    ) -> Self {
        let soft = self
            .soft
            .iter()
            .map(|item| SoftRequestCommitment {
                post: PostId::new(post(item.post.as_str())),
                source: source(&item.source),
                request: item.request,
            })
            .collect();
        Self::new(self.ordinary_tokens, self.hls_tokens, soft)
    }

    pub(super) fn replay_bounded(&self, limit: usize) -> bool {
        self.soft.len() <= limit
    }
}

const fn is_zero(value: &u16) -> bool {
    *value == 0
}
