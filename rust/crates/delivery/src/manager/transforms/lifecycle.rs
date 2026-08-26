use super::{TransformDone, TransformTerminal};
use crate::manager::DeliveryWorker;
use ghostr_engine::adaptive::{DecisionOutcome, ResourceCost};
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::PostId;
use std::collections::HashSet;

impl DeliveryWorker {
    pub(crate) fn finish_transform_job(&mut self, done: &TransformDone) {
        let Some(finish) = self.transforms.finish(done) else {
            return;
        };
        self.state.finish_transform(&finish.post);
        self.resolve_transform_decision(done, finish.cancellation_requested);
        self.request_immediate_replan();
    }

    fn resolve_transform_decision(&self, done: &TransformDone, cancelled: bool) {
        let outcome = outcome(done.terminal, cancelled);
        let observed_at_ms = crate::manager::time::unix_time_ms();
        match done.actual_resources {
            Some(actual) => self.commands.resolve_decision_with_resources(
                done.action,
                outcome,
                resource_cost(actual),
                observed_at_ms,
            ),
            None => self
                .commands
                .resolve_decision(done.action, outcome, observed_at_ms),
        };
    }

    pub(crate) fn cancel_all_transforms(&mut self) {
        self.transforms.clear();
    }

    pub(crate) fn retain_transform_jobs(&mut self, retained: &HashSet<PostId>) {
        self.transforms.retain(retained);
    }

    pub(crate) fn cancel_transform(&mut self, post: &PostId) {
        self.transforms.cancel_post(post);
    }

    pub(crate) fn cancel_obsolete_transform(&mut self, binding: &RepresentationBinding) {
        self.transforms.cancel_obsolete(binding);
    }
}

fn outcome(terminal: TransformTerminal, cancelled: bool) -> DecisionOutcome {
    if cancelled {
        return DecisionOutcome::Cancelled {
            bytes: 0,
            elapsed_ms: 0,
        };
    }
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

fn resource_cost(actual: super::TransformActualResources) -> ResourceCost {
    ResourceCost::new(0, actual.storage_bytes(), actual.cpu_ms(), 0)
}
