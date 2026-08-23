use crate::client_capability::{CapabilityAttempt, ClientCapabilityModel};
use crate::delivery_events::PlayerPreparationReport;
use ghostr_engine::PostId;
use std::collections::HashMap;

pub(super) fn retain_preparations(
    reports: &mut HashMap<PostId, PlayerPreparationReport>,
    capabilities: &mut ClientCapabilityModel,
    mut keep: impl FnMut(&PostId, &PlayerPreparationReport) -> bool,
) {
    reports.retain(|post, report| {
        let retained = keep(post, report);
        if !retained {
            abandon(capabilities, report);
        }
        retained
    });
}

pub(super) fn abandon(capabilities: &mut ClientCapabilityModel, report: &PlayerPreparationReport) {
    capabilities.abandon(
        report.player_capability_generation(),
        CapabilityAttempt::new(report.client_epoch(), report.attempt_generation()),
    );
}
