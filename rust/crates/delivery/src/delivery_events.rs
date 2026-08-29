//! Inbound delivery controls arrive over a channel so the manager reacts to
//! focus, config, and network events instead of polling.

use ghostr_engine::DataUsageLevel;
use tokio::sync::{mpsc, oneshot};

mod channel;
mod decision_log;
mod evidence_snapshot;
mod focus_generation;
#[cfg(test)]
mod generated_focus_api_test;
mod mailbox;
mod models;
mod network;
mod plan_evidence;
mod playback_presentation;
mod player_preparation;
mod receiver;
mod transport;
use crate::evaluation::EvaluationLedger;
use crate::playback_admission::{PlaybackAdmissionLedger, PlaybackAdmissionSnapshot};
pub use channel::command_channel;

pub use decision_log::DecisionHistorySnapshot;
use decision_log::DecisionLog;
pub(crate) use decision_log::{
    DecisionClaim, DecisionPublicationReceipt, DecisionResolution, DecisionToken,
    LegacyDecisionPublication, RequestDecisionBinding, WarpDecisionPublication,
};
pub(crate) use focus_generation::FocusGenerationGuard;
pub use focus_generation::{FocusAdmission, FocusGeneration};
pub use mailbox::MailboxReceiver;
use mailbox::MailboxSender;
pub(crate) use mailbox::PlayerPreparationEnvelope;
pub use models::{
    CandidateAdmission, DeliveryCandidate, DeliveryCommand, DeliveryFocus, FocusItem, FocusPreview,
    FocusTransition,
};
pub use network::DeliveryNetworkStatus;
pub(crate) use network::DeliveryNetworkStatusReader;
pub use plan_evidence::PlanEvidence;
use plan_evidence::PlanEvidenceHistory;
pub(crate) use plan_evidence::PlanPublicationContext;
pub use playback_presentation::{PlaybackPresentation, PlaybackPresentationIngress};
pub(crate) use player_preparation::PlayerPreparationActorOutcome;
pub(crate) use player_preparation::DECODER_UNSUPPORTED_FAILURE;
pub use player_preparation::{
    PlayerPreparationAdmission, PlayerPreparationAttempt, PlayerPreparationAuthority,
    PlayerPreparationClaim, PlayerPreparationDisposition, PlayerPreparationFollowup,
    PlayerPreparationIngress, PlayerPreparationObservation, PlayerPreparationReport,
    PlayerPreparationState,
};
pub use transport::{DeliveryPlayback, TransportRescue, TransportRescueReason};

const DEFAULT_CANDIDATE_CAPACITY: usize = 32;

/// Cloneable control handle. Replaceable controls never block; clear
/// requests apply bounded backpressure. Dropping every clone ends the
/// manager task.
#[derive(Clone, Debug)]
pub struct DeliveryHandle {
    sender: MailboxSender,
    clears: mpsc::Sender<ClearRequest>,
    plans: PlanEvidenceHistory,
    playback_admissions: PlaybackAdmissionLedger,
    evaluation: EvaluationLedger,
    decisions: DecisionLog,
}

impl DeliveryHandle {
    pub fn admit_candidate(&self, candidate: DeliveryCandidate) -> CandidateAdmission {
        self.sender.send_candidate(candidate)
    }

    pub fn update_focus(&self, focus: DeliveryFocus) -> FocusAdmission {
        self.sender.send_focus(focus)
    }

    pub fn report_playback(&self, playback: DeliveryPlayback) {
        self.sender
            .send_control(DeliveryCommand::Playback(playback));
    }

    pub fn set_data_usage(&self, level: DataUsageLevel) {
        self.sender.send_control(DeliveryCommand::Config(level));
    }

    pub fn update_network_profile(
        &self,
        profile: crate::debug::network::NetworkProfile,
    ) -> Option<u64> {
        self.sender.send_network_profile(profile)
    }

    pub fn update_network_status(&self, status: DeliveryNetworkStatus) -> bool {
        self.sender
            .send_control(DeliveryCommand::NetworkStatus(status))
    }

    pub fn storage_changed(&self) {
        self.sender.send_control(DeliveryCommand::StorageChanged);
    }

    pub fn playback_admission_snapshot(&self) -> PlaybackAdmissionSnapshot {
        self.playback_admissions.snapshot()
    }
}

pub type ClearRequest = oneshot::Sender<anyhow::Result<()>>;
pub struct CommandReceiver {
    commands: MailboxReceiver,
    clears: mpsc::Receiver<ClearRequest>,
    plans: PlanEvidenceHistory,
    playback_admissions: PlaybackAdmissionLedger,
    evaluation: EvaluationLedger,
    decisions: DecisionLog,
}

#[cfg(test)]
#[path = "delivery_events_axiom_test.rs"]
pub(crate) mod axiom_test_support;
