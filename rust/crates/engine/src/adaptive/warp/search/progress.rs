use super::output::{PrunedSearchPlan, SearchAudit};
use super::state::{compare, State};
use super::SearchPruneReason;

const MAX_AUDIT_PLANS: usize = 64;

pub(super) struct SearchProgress<'a, F> {
    pub scorer: &'a mut F,
    pub expansions: usize,
    pub best: Option<State>,
    pub(super) retained: Vec<State>,
    pruned: Vec<PrunedSearchPlan>,
    prune_events_total: u64,
    pruned_sample_truncated: bool,
}

impl<'a, F> SearchProgress<'a, F> {
    pub(super) fn new(scorer: &'a mut F) -> Self {
        Self {
            scorer,
            expansions: 0,
            best: None,
            retained: Vec::new(),
            pruned: Vec::new(),
            prune_events_total: 0,
            pruned_sample_truncated: false,
        }
    }

    pub(super) fn retain_best(&mut self, child: &State) {
        self.expansions += 1;
        if child.score > 0
            && self
                .best
                .as_ref()
                .is_none_or(|old| compare(child, old).is_lt())
        {
            self.best = Some(child.clone());
        }
    }

    pub(super) fn prune_state(&mut self, state: State, reason: SearchPruneReason) {
        self.prune(PrunedSearchPlan {
            action_ids: state.sequence.iter().map(|node| node.id).collect(),
            reason,
        });
    }

    pub(super) fn prune(&mut self, plan: PrunedSearchPlan) {
        self.prune_events_total = self.prune_events_total.saturating_add(1);
        if self.pruned.contains(&plan) {
            return;
        }
        if self.pruned.len() < MAX_AUDIT_PLANS {
            self.pruned.push(plan);
        } else {
            self.pruned_sample_truncated = true;
        }
    }

    pub(super) fn audit(&mut self) -> SearchAudit {
        SearchAudit {
            retained: std::mem::take(&mut self.retained),
            pruned: std::mem::take(&mut self.pruned),
            prune_events_total: self.prune_events_total,
            pruned_sample_truncated: self.pruned_sample_truncated,
        }
    }
}
