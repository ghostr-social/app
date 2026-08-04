//! Inbound control surface of the delivery manager: focus and config
//! updates arrive over a channel so the manager reacts to events
//! instead of polling.

use crate::engine::{DataUsageLevel, PostId, VideoMeta};
use tokio::sync::mpsc;

/// One post of the viewer's focus window with its discovery metadata.
#[derive(Clone, Debug)]
pub struct FocusItem {
    pub post: PostId,
    pub meta: VideoMeta,
}

/// A full replacement of the focus window (plan §2 `ffi_update_focus`).
#[derive(Clone, Debug)]
pub struct DeliveryFocus {
    pub items: Vec<FocusItem>,
    pub current_index: usize,
    pub watch_ms: u64,
}

/// Control events the manager reacts to.
#[derive(Clone, Debug)]
pub enum DeliveryCommand {
    Focus(DeliveryFocus),
    Config(DataUsageLevel),
}

/// Cloneable control handle; sends never block. The manager task ends
/// once every handle clone is dropped.
#[derive(Clone, Debug)]
pub struct DeliveryHandle {
    sender: mpsc::UnboundedSender<DeliveryCommand>,
}

impl DeliveryHandle {
    pub fn update_focus(&self, focus: DeliveryFocus) {
        let _ = self.sender.send(DeliveryCommand::Focus(focus));
    }

    pub fn set_data_usage(&self, level: DataUsageLevel) {
        let _ = self.sender.send(DeliveryCommand::Config(level));
    }
}

pub type CommandReceiver = mpsc::UnboundedReceiver<DeliveryCommand>;

pub fn command_channel() -> (DeliveryHandle, CommandReceiver) {
    let (sender, receiver) = mpsc::unbounded_channel();
    (DeliveryHandle { sender }, receiver)
}
