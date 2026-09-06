use super::super::allocation::resources;
use super::super::builder::{Builder, NodeInput};
use super::super::{GeneratedAction, PlannerCommand};
use crate::adaptive::{ActionKind, CandidateSnapshot, HeadProbeHistory};

impl Builder<'_> {
    pub(super) fn add_head(&mut self, candidate: &CandidateSnapshot) {
        let head_suppressed = self
            .context
            .candidate(&candidate.post)
            .is_some_and(|item| item.head_probe != HeadProbeHistory::Unobserved);
        let current = candidate.post == self.snapshot.playback.current;
        let in_window = self
            .generation_policies
            .navigation
            .includes_knowledge(candidate.feed_offset);
        if !candidate.needs_bootstrap() || current || head_suppressed || !in_window {
            return;
        }
        let kind = ActionKind::Head;
        let Some(source) = self.optional_exploration_source(candidate, &kind) else {
            return;
        };
        let prediction = self.prediction(candidate, &kind, source);
        let input = NodeInput::new(kind.clone(), source, prediction, &[]).optional_exploration();
        let mut node = self.node(candidate, input);
        node.resources = resources(&kind);
        self.actions.push(GeneratedAction {
            node,
            command: PlannerCommand::ProbeHead {
                post: candidate.post.clone(),
                source: source.to_owned(),
                authority: super::super::allocation::authority(
                    candidate,
                    self.snapshot,
                    self.base.mode,
                ),
            },
        });
    }
}
