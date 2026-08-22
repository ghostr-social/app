mod range_actions;

use super::allocation::{classify, resources, AllocationSpec};
use super::builder::{Builder, NodeInput};
use super::prediction::transform_prediction;
use super::{GeneratedAction, PlannerCommand};
use crate::adaptive::{ActionKind, CandidateSnapshot, MediaLayout};

impl Builder<'_> {
    pub(super) fn add_candidate(&mut self, candidate: &CandidateSnapshot) {
        self.add_head(candidate);
        self.add_base(candidate);
        if candidate.layout != MediaLayout::RequiresCompleteFile {
            self.add_range_actions(candidate);
        }
        let whole = self.add_whole(candidate);
        self.add_transform(candidate, whole);
        self.add_active(candidate);
    }

    fn add_head(&mut self, candidate: &CandidateSnapshot) {
        let Some(source) = self.request_source(candidate) else {
            return;
        };
        if crate::adaptive::ranges::body_complete(candidate) {
            return;
        }
        let head_suppressed = self
            .context
            .candidate(&candidate.post)
            .is_some_and(|item| item.head_probe != super::super::HeadProbeHistory::Unobserved);
        if !candidate.needs_bootstrap() || head_suppressed {
            return;
        }
        let kind = ActionKind::Head;
        let prediction = self.prediction(candidate, &kind, source);
        let input = NodeInput::new(kind.clone(), source, prediction, &[]);
        let mut node = self.node(candidate, input);
        node.resources = resources(&kind);
        self.actions.push(GeneratedAction {
            node,
            command: PlannerCommand::ProbeHead {
                post: candidate.post.clone(),
                source: source.to_owned(),
                authority: super::allocation::authority(candidate, self.snapshot, self.base.mode),
            },
        });
    }

    fn add_base(&mut self, candidate: &CandidateSnapshot) {
        if !self.permits_request(candidate) {
            return;
        }
        let allocations: Vec<_> = self
            .base
            .allocations
            .iter()
            .filter(|item| item.post == candidate.post)
            .cloned()
            .collect();
        for allocation in allocations {
            let allocation = self.normalize_playability(candidate, allocation);
            self.push_transfer(candidate, classify(&allocation), allocation, &[]);
        }
    }

    fn add_whole(&mut self, candidate: &CandidateSnapshot) -> Option<u16> {
        let (Some(total), Some(source)) = (candidate.total_bytes, self.request_source(candidate))
        else {
            return None;
        };
        if candidate.finalized || crate::adaptive::ranges::body_complete(candidate) {
            return None;
        }
        let kind = ActionKind::FetchWhole {
            maximum_bytes: total,
        };
        if self.contains(candidate, &kind) {
            return self.action_id(candidate, |item| {
                matches!(item, ActionKind::FetchWhole { .. })
            });
        }
        let allocation = self.allocation(
            candidate,
            AllocationSpec::whole(total, source, candidate.duration_ms),
        );
        Some(self.push_transfer(candidate, kind, allocation, &[]))
    }

    fn add_transform(&mut self, candidate: &CandidateSnapshot, whole: Option<u16>) {
        let Some(transform) = self
            .context
            .candidate(&candidate.post)
            .and_then(|item| item.capability.required_transform())
        else {
            return;
        };
        let kind = ActionKind::Transform(transform.kind);
        let requires: Vec<_> = whole.into_iter().collect();
        let prediction = transform_prediction(candidate, transform.estimated_cpu_ms);
        let input = NodeInput::new(kind, "local-transform", prediction, &requires);
        let mut node = self.node(candidate, input);
        node.resources = super::super::ResourceCost::new(
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
