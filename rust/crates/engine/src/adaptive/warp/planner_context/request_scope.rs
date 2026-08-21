use crate::adaptive::{ActionNode, RetrievalRequest};
use crate::PostId;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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

#[derive(Clone, Debug)]
pub(super) struct RequestScope {
    ordinary_tokens: u16,
    soft: Vec<SoftRequestCommitment>,
}

impl RequestScope {
    pub(super) fn new(ordinary_tokens: u16, soft: Vec<SoftRequestCommitment>) -> Self {
        Self {
            ordinary_tokens,
            soft,
        }
    }

    pub(super) fn admits(&self, action: &ActionNode, occupied: usize) -> bool {
        occupied < usize::from(self.ordinary_tokens)
            || self.soft.iter().any(|item| item.admits(action))
    }
}
