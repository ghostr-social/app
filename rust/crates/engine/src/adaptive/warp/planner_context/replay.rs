use super::PlannerContext;
use std::collections::BTreeMap;

mod candidate_serde;

const ENTRY_LIMIT: usize = 64;

impl PlannerContext {
    pub(crate) fn replay_sources(&self) -> Vec<String> {
        let mut values: Vec<_> = self
            .active
            .values()
            .filter_map(|item| item.replay_source())
            .collect();
        if let Some(scope) = &self.request_scope {
            values.extend(scope.replay_sources());
        }
        values
    }

    pub(crate) fn replay_project(
        &self,
        post: &impl Fn(&str) -> String,
        source: &impl Fn(&str) -> String,
    ) -> Self {
        let mut projected = self.clone();
        projected.candidates = self
            .candidates
            .iter()
            .map(|(id, value)| (crate::PostId::new(post(id.as_str())), *value))
            .collect();
        projected.active = project_active(&self.active, post, source);
        projected.request_occupancy = self.request_occupancy.replay_project(source);
        projected.request_scope = self
            .request_scope
            .as_ref()
            .map(|scope| scope.replay_project(post, source));
        projected
    }

    pub(crate) fn replay_bounded(&self) -> bool {
        self.candidates.len() <= ENTRY_LIMIT
            && self.active.len() <= ENTRY_LIMIT
            && self.request_occupancy.replay_bounded(ENTRY_LIMIT)
            && self
                .request_scope
                .as_ref()
                .is_none_or(|scope| scope.replay_bounded(ENTRY_LIMIT))
    }
}

fn project_active(
    values: &BTreeMap<crate::ActionId, super::ActivePlannerContext>,
    post: &impl Fn(&str) -> String,
    source: &impl Fn(&str) -> String,
) -> BTreeMap<crate::ActionId, super::ActivePlannerContext> {
    values
        .iter()
        .map(|(id, value)| (*id, value.replay_project(post, source)))
        .collect()
}
