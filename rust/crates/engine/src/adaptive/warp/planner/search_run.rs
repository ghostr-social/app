use super::{
    feasibility, least_risk, reserve_progress, simulation, ReserveProgressPolicy,
    SearchReplayInput, SearchReplayMode, WarpPlanner, WarpPlannerInput,
};
use crate::adaptive::warp::ScoredSearchPlan;
use crate::adaptive::{ReserveConstraint, SearchDecision, TwinSearchContext, WarpSearch};

#[cfg(test)]
#[path = "search_run/protected_reserve_selection_test.rs"]
mod protected_reserve_selection_test;

impl WarpPlanner {
    pub(super) fn search(
        &mut self,
        input: &WarpPlannerInput<'_>,
        feasible: &feasibility::FeasibleActions,
    ) -> (SearchDecision, SearchReplayInput) {
        if let Some(demanded) = super::demand::constrain(input, feasible) {
            let search = least_risk::choose_with_reason(
                &demanded.nodes,
                &[],
                crate::adaptive::SearchPruneReason::DemandedInput,
            );
            let replay = self.replay_input(&demanded, &search, Vec::new(), Vec::new());
            return (search, replay);
        }
        let progress =
            if self.config.reserve_progress_policy == ReserveProgressPolicy::OrderedReadiness {
                reserve_progress::action_ids(input.snapshot, input.base, &feasible.nodes)
            } else {
                Vec::new()
            };
        if feasible.reserve.degraded || !progress.is_empty() {
            let preferred = protected_progress_ids(&feasible.reserve, progress);
            let search = least_risk::choose(&feasible.nodes, &preferred);
            let replay = self.replay_input(feasible, &search, Vec::new(), preferred);
            return (search, replay);
        }
        if self.config.profile == super::PlannerProfile::Core3 {
            let search =
                WarpSearch::new(self.config.beam).choose_core(&feasible.nodes, &feasible.budget);
            let replay = self.replay_input(feasible, &search, Vec::new(), Vec::new());
            return (search, replay);
        }
        self.search_priced(input, feasible)
    }

    fn search_priced(
        &mut self,
        input: &WarpPlannerInput<'_>,
        feasible: &feasibility::FeasibleActions,
    ) -> (SearchDecision, SearchReplayInput) {
        let (search, scores) = self.run_priced_search(input, feasible);
        let replay = self.replay_input(feasible, &search, scores, Vec::new());
        (search, replay)
    }

    fn run_priced_search(
        &mut self,
        input: &WarpPlannerInput<'_>,
        feasible: &feasibility::FeasibleActions,
    ) -> (SearchDecision, Vec<ScoredSearchPlan>) {
        let mut simulation = TwinSearchContext::new(
            &mut self.twin,
            simulation::state(input),
            simulation::epochs(input, self.price_epoch),
        );
        let search = WarpSearch::new(self.config.beam)
            .with_prices(self.prices.prices())
            .choose_first_simulated(&feasible.nodes, feasible.budget.clone(), &mut simulation);
        (search, simulation.scored_plans().to_vec())
    }

    fn replay_input(
        &self,
        feasible: &feasibility::FeasibleActions,
        search: &SearchDecision,
        scores: Vec<ScoredSearchPlan>,
        reserve_progress_action_ids: Vec<u16>,
    ) -> SearchReplayInput {
        let mode = SearchReplayMode::capture(
            search,
            feasible.reserve.degraded,
            !reserve_progress_action_ids.is_empty(),
        );
        SearchReplayInput {
            mode,
            reserve: feasible.reserve.clone(),
            reserve_threshold_bps: feasible.reserve.chance.map(|chance| chance.threshold_bps),
            reserve_degraded_reason: feasible.reserve.degraded_reason,
            nodes: feasible.nodes.clone(),
            budget: feasible.budget.clone(),
            beam: self.config.beam,
            prices: self.prices.prices(),
            scores: if mode == SearchReplayMode::Beam {
                scores
            } else {
                Vec::new()
            },
            reserve_progress_action_ids,
        }
    }
}

fn protected_progress_ids(reserve: &ReserveConstraint, discovered: Vec<u16>) -> Vec<u16> {
    if reserve.degraded {
        discovered
    } else {
        reserve.protected_action_ids.clone()
    }
}
