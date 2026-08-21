use super::{
    feasibility, least_risk, simulation, SearchReplayInput, SearchReplayMode, WarpPlanner,
    WarpPlannerInput,
};
use crate::adaptive::warp::ScoredSearchPlan;
use crate::adaptive::{SearchDecision, TwinSearchContext, WarpSearch};

impl WarpPlanner {
    pub(super) fn search(
        &mut self,
        input: &WarpPlannerInput<'_>,
        feasible: &feasibility::FeasibleActions,
    ) -> (SearchDecision, SearchReplayInput) {
        if feasible.reserve.degraded {
            let search = least_risk::choose(&feasible.nodes);
            let replay = self.replay_input(feasible, &search, Vec::new());
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
        let replay = self.replay_input(feasible, &search, scores);
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
    ) -> SearchReplayInput {
        let mode = SearchReplayMode::capture(search, feasible.reserve.degraded);
        SearchReplayInput {
            mode,
            nodes: feasible.nodes.clone(),
            budget: feasible.budget.clone(),
            beam: self.config.beam,
            prices: self.prices.prices(),
            scores: if mode == SearchReplayMode::Beam {
                scores
            } else {
                Vec::new()
            },
        }
    }
}
