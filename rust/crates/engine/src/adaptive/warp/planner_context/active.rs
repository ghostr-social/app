use super::super::{HedgeInput, IdentityProof};
use crate::{ActionId, PostId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct HedgeContext {
    input: HedgeInput,
    identity: IdentityProof,
    alternate: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ActivePlannerContext {
    pub(crate) action: ActionId,
    pub(crate) continuation_advantage_micros: Option<i64>,
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
            .map(|item| (&item.input, item.identity, item.alternate.as_str()))
    }

    pub(super) fn replay_source(&self) -> Option<String> {
        self.hedge.as_ref().map(|hedge| hedge.alternate.clone())
    }

    pub(super) fn replay_project(
        &self,
        post: &impl Fn(&str) -> String,
        source: &impl Fn(&str) -> String,
    ) -> Self {
        let mut value = self.clone();
        value.post = PostId::new(post(value.post.as_str()));
        if let Some(hedge) = &mut value.hedge {
            hedge.alternate = source(&hedge.alternate);
            project_kind(&mut hedge.input.action, source);
        }
        value
    }
}

fn project_kind(kind: &mut super::super::ActionKind, source: &impl Fn(&str) -> String) {
    if let super::super::ActionKind::Hedge { alternate, .. } = kind {
        *alternate = source(alternate);
    }
}
