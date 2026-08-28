use super::super::builder::{Builder, NodeInput};
use super::super::prediction::transform_prediction;
use super::super::{GeneratedAction, PlannerCommand};
use crate::adaptive::{ActionKind, CandidateSnapshot, ResourceCost};

impl Builder<'_> {
    pub(super) fn add_transform(&mut self, candidate: &CandidateSnapshot, whole: Option<u16>) {
        let Some(transform) = self
            .context
            .candidate(&candidate.post)
            .and_then(|item| item.capability.required_transform())
        else {
            return;
        };
        if !candidate.finalized && whole.is_none() {
            return;
        }
        let kind = ActionKind::Transform(transform.kind);
        let requires: Vec<_> = whole.into_iter().collect();
        let prediction = transform_prediction(candidate, transform.estimated_cpu_ms);
        let input = NodeInput::new(kind, "local-transform", prediction, &requires);
        let mut node = self.node(candidate, input);
        node.resources = ResourceCost::new(
            0,
            transform.output_upper_bytes,
            transform.estimated_cpu_ms,
            0,
        );
        self.actions.push(GeneratedAction {
            node,
            command: PlannerCommand::Transform {
                post: candidate.post.clone(),
                kind: transform.kind,
            },
        });
    }
}
