//! Inbound control surface of the delivery manager: focus, config, and
//! network updates arrive over a channel so the manager reacts to events
//! instead of polling.

use ghostr_engine::evidence::NostrMetadataEvidence;
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::{DataUsageLevel, PostId, PreviewDescriptor, VideoMeta};
use tokio::sync::{mpsc, oneshot};

mod channel;
mod decision_log;
mod focus_generation;
mod mailbox;
mod plan_evidence;
mod playback_presentation;
mod player_preparation;
mod receiver;
mod transport;
use crate::evaluation::{EvaluationLedger, EvaluationSnapshot};
use crate::playback_admission::{PlaybackAdmissionLedger, PlaybackAdmissionSnapshot};
pub use channel::{command_channel, command_channel_with_candidate_capacity};
pub use decision_log::DecisionHistorySnapshot;
use decision_log::DecisionLog;
pub(crate) use decision_log::{
    DecisionClaim, DecisionResolution, DecisionToken, LegacyDecisionPublication,
    RequestDecisionBinding, WarpDecisionPublication,
};
pub(crate) use focus_generation::FocusGenerationGuard;
pub use focus_generation::{FocusAdmission, FocusGeneration};
pub use mailbox::MailboxReceiver;
use mailbox::MailboxSender;
pub use plan_evidence::PlanEvidence;
use plan_evidence::PlanEvidenceHistory;
pub use playback_presentation::{PlaybackPresentation, PlaybackPresentationIngress};
pub use player_preparation::{
    PlayerPreparationAttempt, PlayerPreparationAuthority, PlayerPreparationIngress,
    PlayerPreparationObservation, PlayerPreparationReport, PlayerPreparationState,
};
pub use transport::{DeliveryPlayback, TransportRescue, TransportRescueReason};

const DEFAULT_CANDIDATE_CAPACITY: usize = 32;

/// One post of the viewer's focus window with its discovery metadata.
#[derive(Clone, Debug)]
pub struct FocusItem {
    pub post: PostId,
    pub meta: VideoMeta,
}

/// A validated discovery candidate available to probes before focus ranks it.
#[derive(Clone, Debug)]
pub struct DeliveryCandidate {
    pub post: PostId,
    pub meta: VideoMeta,
    pub preview: Option<PreviewDescriptor>,
    pub metadata_evidence: Vec<NostrMetadataEvidence>,
    pub renditions: Vec<VideoRendition>,
    pub discovered_at: u64,
}

/// Validated inline preview evidence associated with one focused post.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FocusPreview {
    pub post: PostId,
    pub descriptor: PreviewDescriptor,
}

/// Whether a focus movement represents user navigation or system control.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FocusTransition {
    UserNavigation,
    RosterChange,
    TransportRescue,
}

/// A full replacement of the focus window (plan §2 `ffi_update_focus`).
#[derive(Clone, Debug)]
pub struct DeliveryFocus {
    pub items: Vec<FocusItem>,
    pub previews: Vec<FocusPreview>,
    pub current_index: usize,
    pub watch_ms: u64,
    pub generation: FocusGeneration,
    pub transition: FocusTransition,
    pub rescue: Option<TransportRescue>,
}

impl DeliveryFocus {
    pub fn compatibility(items: Vec<FocusItem>, current_index: usize, watch_ms: u64) -> Self {
        Self {
            items,
            previews: Vec::new(),
            current_index,
            watch_ms,
            generation: FocusGeneration::compatibility(),
            transition: FocusTransition::UserNavigation,
            rescue: None,
        }
    }
}

/// Control events the manager reacts to.
#[derive(Debug)]
pub enum DeliveryCommand {
    Candidate(DeliveryCandidate),
    Focus(DeliveryFocus),
    Playback(DeliveryPlayback),
    Config(DataUsageLevel),
    NetworkChanged,
    StorageChanged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidateAdmission {
    Accepted,
    Saturated,
    Closed,
}

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

    pub fn network_changed(&self) {
        self.sender.send_control(DeliveryCommand::NetworkChanged);
    }

    pub fn storage_changed(&self) {
        self.sender.send_control(DeliveryCommand::StorageChanged);
    }

    pub fn playback_admission_snapshot(&self) -> PlaybackAdmissionSnapshot {
        self.playback_admissions.snapshot()
    }

    pub fn evaluation_snapshot(&self) -> EvaluationSnapshot {
        self.evaluation.snapshot()
    }

    pub fn decision_history(&self) -> DecisionHistorySnapshot {
        self.decisions.snapshot()
    }

    pub async fn clear(&self) -> anyhow::Result<()> {
        let (reply, result) = oneshot::channel();
        self.clears
            .send(reply)
            .await
            .map_err(|_| anyhow::anyhow!("delivery manager is unavailable"))?;
        result
            .await
            .map_err(|_| anyhow::anyhow!("delivery reset was interrupted"))?
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
