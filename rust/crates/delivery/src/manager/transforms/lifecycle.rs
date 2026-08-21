use super::{TransformDone, TransformTerminal};
use crate::manager::DeliveryWorker;
use ghostr_engine::adaptive::DecisionOutcome;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{ActionId, PostId};
use std::collections::HashSet;

impl DeliveryWorker {
    pub(crate) fn finish_transform_job(&mut self, done: TransformDone) {
        let Some(post) = self.transforms.finish(done.action) else {
            return;
        };
        self.state.finish_transform(&post);
        self.commands.resolve_decision(
            done.action,
            outcome(done.terminal),
            crate::manager::time::unix_time_ms(),
        );
        self.request_immediate_replan();
    }

    pub(crate) fn cancel_all_transforms(&mut self) {
        let cancelled = self.transforms.clear();
        self.resolve_transform_cancellations(cancelled);
    }

    pub(crate) fn retain_transform_jobs(&mut self, retained: &HashSet<PostId>) {
        let cancelled = self.transforms.retain(retained);
        self.resolve_transform_cancellations(cancelled);
    }

    pub(crate) fn cancel_obsolete_transform(&mut self, binding: &RepresentationBinding) {
        let cancelled = self.transforms.cancel_obsolete(binding);
        self.resolve_transform_cancellations(cancelled);
    }

    fn resolve_transform_cancellations(&mut self, cancelled: Vec<(ActionId, PostId)>) {
        for (action, post) in cancelled {
            self.state.finish_transform(&post);
            self.commands.resolve_decision(
                action,
                DecisionOutcome::Cancelled {
                    bytes: 0,
                    elapsed_ms: 0,
                },
                crate::manager::time::unix_time_ms(),
            );
        }
    }
}

fn outcome(terminal: TransformTerminal) -> DecisionOutcome {
    match terminal {
        TransformTerminal::Succeeded(bytes) => DecisionOutcome::Succeeded {
            bytes,
            elapsed_ms: 0,
        },
        TransformTerminal::Failed(class) => DecisionOutcome::Failed {
            class: class.to_owned(),
            elapsed_ms: 0,
        },
    }
}
