//! Inbound control surface of the delivery manager: focus and config
//! updates arrive over a channel so the manager reacts to events
//! instead of polling.

use ghostr_engine::{DataUsageLevel, PostId, VideoMeta};
use tokio::sync::{mpsc, oneshot};

/// One post of the viewer's focus window with its discovery metadata.
#[derive(Clone, Debug)]
pub struct FocusItem {
    pub post: PostId,
    pub meta: VideoMeta,
}

/// A validated discovery candidate. Admission makes metadata available
/// to probing immediately; feed focus only changes its download rank.
#[derive(Clone, Debug)]
pub struct DeliveryCandidate {
    pub post: PostId,
    pub meta: VideoMeta,
    pub discovered_at: u64,
}

/// A full replacement of the focus window (plan §2 `ffi_update_focus`).
#[derive(Clone, Debug)]
pub struct DeliveryFocus {
    pub items: Vec<FocusItem>,
    pub current_index: usize,
    pub watch_ms: u64,
}

/// Control events the manager reacts to.
#[derive(Debug)]
pub enum DeliveryCommand {
    Candidate(DeliveryCandidate),
    Prioritize(PostId),
    Focus(DeliveryFocus),
    Config(DataUsageLevel),
}

/// Cloneable control handle; sends never block. The manager task ends
/// once every handle clone is dropped.
#[derive(Clone, Debug)]
pub struct DeliveryHandle {
    sender: mpsc::UnboundedSender<DeliveryCommand>,
    clears: mpsc::UnboundedSender<ClearRequest>,
}

impl DeliveryHandle {
    pub fn admit_candidate(&self, candidate: DeliveryCandidate) {
        let _ = self.sender.send(DeliveryCommand::Candidate(candidate));
    }

    pub fn prioritize_candidate(&self, post: PostId) {
        let _ = self.sender.send(DeliveryCommand::Prioritize(post));
    }

    pub fn update_focus(&self, focus: DeliveryFocus) {
        let _ = self.sender.send(DeliveryCommand::Focus(focus));
    }

    pub fn set_data_usage(&self, level: DataUsageLevel) {
        let _ = self.sender.send(DeliveryCommand::Config(level));
    }

    pub async fn clear(&self) -> anyhow::Result<()> {
        let (reply, result) = oneshot::channel();
        self.clears
            .send(reply)
            .map_err(|_| anyhow::anyhow!("delivery manager is unavailable"))?;
        result
            .await
            .map_err(|_| anyhow::anyhow!("delivery reset was interrupted"))?
    }
}

pub type ClearRequest = oneshot::Sender<anyhow::Result<()>>;

pub struct CommandReceiver {
    commands: mpsc::UnboundedReceiver<DeliveryCommand>,
    clears: mpsc::UnboundedReceiver<ClearRequest>,
}

impl CommandReceiver {
    pub async fn recv(&mut self) -> Option<DeliveryCommand> {
        self.commands.recv().await
    }

    pub(crate) fn receivers(
        &mut self,
    ) -> (
        &mut mpsc::UnboundedReceiver<DeliveryCommand>,
        &mut mpsc::UnboundedReceiver<ClearRequest>,
    ) {
        (&mut self.commands, &mut self.clears)
    }

    pub(crate) fn discard_pending(&mut self) {
        while self.commands.try_recv().is_ok() {}
    }
}

pub fn command_channel() -> (DeliveryHandle, CommandReceiver) {
    let (sender, commands) = mpsc::unbounded_channel();
    let (clear_sender, clears) = mpsc::unbounded_channel();
    (
        DeliveryHandle {
            sender,
            clears: clear_sender,
        },
        CommandReceiver { commands, clears },
    )
}
