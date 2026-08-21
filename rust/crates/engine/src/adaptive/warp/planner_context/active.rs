use super::super::{HedgeInput, IdentityProof};
use crate::{ActionId, PostId};

#[derive(Clone, Debug, Eq, PartialEq)]
struct HedgeContext {
    input: HedgeInput,
    identity: IdentityProof,
    alternate: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivePlannerContext {
    pub action: ActionId,
    pub continuation_advantage_micros: Option<i64>,
    post: PostId,
    cancelling: bool,
    hedge: Option<HedgeContext>,
}

impl ActivePlannerContext {
    pub fn new(action: ActionId, post: PostId) -> Self {
        Self {
            action,
            continuation_advantage_micros: None,
            post,
            cancelling: false,
            hedge: None,
        }
    }

    pub const fn mark_cancelling(mut self) -> Self {
        self.cancelling = true;
        self
    }

    pub const fn with_continuation_advantage(mut self, value_micros: i64) -> Self {
        self.continuation_advantage_micros = Some(value_micros);
        self
    }

    pub fn with_hedge(
        mut self,
        input: HedgeInput,
        identity: IdentityProof,
        alternate: impl Into<String>,
    ) -> Self {
        self.hedge = Some(HedgeContext {
            input,
            identity,
            alternate: alternate.into(),
        });
        self
    }

    pub(in crate::adaptive::warp) fn post(&self) -> &PostId {
        &self.post
    }

    pub(in crate::adaptive::warp) const fn cancelling(&self) -> bool {
        self.cancelling
    }

    pub(in crate::adaptive::warp) fn hedge(&self) -> Option<(&HedgeInput, IdentityProof, &str)> {
        self.hedge
            .as_ref()
            .map(|item| (&item.input, item.identity.clone(), item.alternate.as_str()))
    }
}
