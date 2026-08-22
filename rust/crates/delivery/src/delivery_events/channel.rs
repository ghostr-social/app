use super::{mailbox, CommandReceiver, DeliveryHandle, DEFAULT_CANDIDATE_CAPACITY};
use crate::playback_admission::PlaybackAdmissionLedger;
use tokio::sync::mpsc;

pub fn command_channel() -> (DeliveryHandle, CommandReceiver) {
    command_channel_with_candidate_capacity(DEFAULT_CANDIDATE_CAPACITY)
}

pub fn command_channel_with_candidate_capacity(
    capacity: usize,
) -> (DeliveryHandle, CommandReceiver) {
    let (sender, commands) = mailbox::channel(capacity);
    let (clear_sender, clears) = mpsc::channel(1);
    let plans = super::PlanEvidenceHistory::default();
    let playback_admissions = PlaybackAdmissionLedger::default();
    let evaluation = crate::evaluation::EvaluationLedger::default();
    let decisions = super::DecisionLog::default();
    (
        DeliveryHandle {
            sender,
            clears: clear_sender,
            plans: plans.clone(),
            playback_admissions: playback_admissions.clone(),
            evaluation: evaluation.clone(),
            decisions: decisions.clone(),
        },
        CommandReceiver {
            commands,
            clears,
            plans,
            playback_admissions,
            evaluation,
            decisions,
        },
    )
}
