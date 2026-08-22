use crate::adaptive::{ActionNode, DigitalTwin, TwinEpochs, TwinState};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ScoredSearchPlan {
    pub action_ids: Vec<u16>,
    pub score_micros: i64,
}

pub struct TwinSearchContext<'a> {
    twin: &'a mut DigitalTwin,
    state: TwinState,
    epochs: TwinEpochs,
    scores: Vec<ScoredSearchPlan>,
}

impl<'a> TwinSearchContext<'a> {
    pub fn new(twin: &'a mut DigitalTwin, state: TwinState, epochs: TwinEpochs) -> Self {
        Self {
            twin,
            state,
            epochs,
            scores: Vec::new(),
        }
    }

    pub(super) fn score(&mut self, actions: &[ActionNode]) -> i64 {
        let score = self
            .twin
            .evaluate(&self.state, actions, self.epochs)
            .expected_score_micros;
        self.scores.push(ScoredSearchPlan {
            action_ids: actions.iter().map(|action| action.id).collect(),
            score_micros: score,
        });
        score
    }

    pub(crate) fn scored_plans(&self) -> &[ScoredSearchPlan] {
        &self.scores
    }
}
