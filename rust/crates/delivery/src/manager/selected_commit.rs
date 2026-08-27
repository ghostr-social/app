//! Exactly-once ownership of the singular action selected by one WARP pass.

use crate::manager::DeliveryWorker;
use ghostr_engine::adaptive::{GeneratedAction, ResourceCost, WarpPlanner};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CommitResult {
    Untracked,
    Committed,
    Rejected,
}

pub(crate) struct SelectedCommit {
    action: Option<GeneratedAction>,
}

impl SelectedCommit {
    pub(crate) const fn new(action: GeneratedAction) -> Self {
        Self {
            action: Some(action),
        }
    }

    pub(super) fn optional(action: Option<GeneratedAction>) -> Option<Self> {
        action.map(Self::new)
    }

    pub(super) fn resources(&self) -> Option<(ResourceCost, ResourceCost)> {
        let action = self.action.as_ref()?;
        Some((action.node.resources, action.node.authorized_resources()))
    }

    pub(super) fn request_profile(
        &self,
    ) -> Option<ghostr_engine::origin_model::OriginRequestProfile> {
        self.action.as_ref()?.node.request_profile()
    }

    pub(crate) fn commit(
        &mut self,
        planner: &mut WarpPlanner,
        resources: ResourceCost,
        observed_at_ms: u64,
    ) -> CommitResult {
        let Some(action) = self.action.take() else {
            return CommitResult::Untracked;
        };
        if planner.commit(&action, resources, observed_at_ms) {
            CommitResult::Committed
        } else {
            CommitResult::Rejected
        }
    }
}

impl DeliveryWorker {
    pub(super) fn commit_selected(
        &mut self,
        selected: &mut Option<SelectedCommit>,
        resources: ResourceCost,
        observed_at_ms: u64,
    ) -> CommitResult {
        let Some(mut selected) = selected.take() else {
            return CommitResult::Untracked;
        };
        selected.commit(&mut self.warp_planner, resources, observed_at_ms)
    }
}
