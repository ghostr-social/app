use super::PlanMetrics;
use crate::adaptive::ActionKind;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RetrievalRung {
    Metadata,
    Preview,
    FirstFrame,
    ReadyPlayback,
    Complete,
    Remuxed,
    Transcoded,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetrievalPlan {
    id: String,
    pub terminal: RetrievalRung,
    pub(crate) actions: Vec<ActionKind>,
    pub(crate) metrics: PlanMetrics,
}

impl RetrievalPlan {
    pub(crate) fn new(
        id: impl Into<String>,
        terminal: RetrievalRung,
        metrics: PlanMetrics,
    ) -> Self {
        Self {
            id: id.into(),
            terminal,
            actions: Vec::new(),
            metrics,
        }
    }

    pub(crate) fn with_actions(mut self, actions: Vec<ActionKind>) -> Self {
        self.actions = actions;
        self
    }

    pub(super) fn id(&self) -> &str {
        &self.id
    }
}
