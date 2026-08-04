use crate::engine::{ByteRange, PostId};
use tokio::sync::mpsc;

/// A playback-driven request for bytes the gateway could not serve yet.
/// The delivery manager consumes it as the T0 promotion signal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DemandSignal {
    pub post: PostId,
    pub range: ByteRange,
}

/// Sending half handed to the gateway; emissions never block and are
/// silently dropped once the consuming manager goes away.
#[derive(Clone, Debug)]
pub struct DemandSender(mpsc::UnboundedSender<DemandSignal>);

impl DemandSender {
    pub fn emit(&self, signal: DemandSignal) {
        let _ = self.0.send(signal);
    }
}

pub type DemandReceiver = mpsc::UnboundedReceiver<DemandSignal>;

pub fn demand_channel() -> (DemandSender, DemandReceiver) {
    let (sender, receiver) = mpsc::unbounded_channel();
    (DemandSender(sender), receiver)
}
