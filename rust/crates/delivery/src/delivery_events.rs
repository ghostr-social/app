//! Inbound control surface of the delivery manager: focus, config, and
//! network updates arrive over a channel so the manager reacts to events
//! instead of polling.

use ghostr_engine::adaptive::AllocationPlan;
use ghostr_engine::playback::{PlaybackObservation, PlaybackObservationSequence, PlaybackSession};
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::{DataUsageLevel, PostId, VideoMeta};
use tokio::sync::{mpsc, oneshot};

mod focus_generation;
mod mailbox;
mod plan_evidence;
pub(crate) use focus_generation::FocusGenerationGuard;
pub use focus_generation::{FocusAdmission, FocusGeneration};
pub use mailbox::MailboxReceiver;
use mailbox::MailboxSender;
pub use plan_evidence::PlanEvidence;
use plan_evidence::PlanEvidenceHistory;

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
    pub renditions: Vec<VideoRendition>,
    pub discovered_at: u64,
}

/// A full replacement of the focus window (plan §2 `ffi_update_focus`).
#[derive(Clone, Debug)]
pub struct DeliveryFocus {
    pub items: Vec<FocusItem>,
    pub current_index: usize,
    pub watch_ms: u64,
    pub generation: FocusGeneration,
}

impl DeliveryFocus {
    pub fn compatibility(items: Vec<FocusItem>, current_index: usize, watch_ms: u64) -> Self {
        Self {
            items,
            current_index,
            watch_ms,
            generation: FocusGeneration::compatibility(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryPlayback {
    pub session: PlaybackSession,
    pub sequence: PlaybackObservationSequence,
    pub observation: PlaybackObservation,
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

    pub fn plan_history(&self) -> Vec<PlanEvidence> {
        self.plans.snapshot()
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
}

impl CommandReceiver {
    pub fn receivers(&mut self) -> (&mut MailboxReceiver, &mut mpsc::Receiver<ClearRequest>) {
        (&mut self.commands, &mut self.clears)
    }

    pub(crate) fn discard_pending(&mut self) {
        self.commands.clear();
    }

    pub(crate) fn try_clear(&mut self) -> Option<ClearRequest> {
        self.clears.try_recv().ok()
    }

    pub(crate) fn has_control(&self) -> bool {
        self.commands.has_control()
    }

    pub(crate) fn has_candidate(&self) -> bool {
        self.commands.has_candidate()
    }

    pub fn try_control(&mut self) -> Option<DeliveryCommand> {
        self.commands.try_control()
    }

    pub fn try_candidate(&mut self) -> Option<DeliveryCandidate> {
        self.commands.try_candidate()
    }

    pub fn publish_plan(&mut self, observed_at_ms: u64, plan: AllocationPlan) {
        self.plans.publish(observed_at_ms, plan);
    }
}

pub fn command_channel() -> (DeliveryHandle, CommandReceiver) {
    command_channel_with_candidate_capacity(DEFAULT_CANDIDATE_CAPACITY)
}

pub fn command_channel_with_candidate_capacity(
    capacity: usize,
) -> (DeliveryHandle, CommandReceiver) {
    let (sender, commands) = mailbox::channel(capacity);
    let (clear_sender, clears) = mpsc::channel(1);
    let plans = PlanEvidenceHistory::default();
    (
        DeliveryHandle {
            sender,
            clears: clear_sender,
            plans: plans.clone(),
        },
        CommandReceiver {
            commands,
            clears,
            plans,
        },
    )
}
