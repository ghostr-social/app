use crate::manager::transfers::InternalEvent;
use crate::manager::DeliveryWorker;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Default)]
pub(crate) struct ImmediateReplan {
    pending: bool,
}

impl ImmediateReplan {
    pub(crate) fn request(&mut self, events: &UnboundedSender<InternalEvent>) -> bool {
        if self.pending {
            return false;
        }
        self.pending = true;
        if events.send(InternalEvent::ImmediateReplan).is_ok() {
            return true;
        }
        self.pending = false;
        false
    }

    pub(crate) fn consume(&mut self) {
        self.pending = false;
    }
}

impl DeliveryWorker {
    pub(super) fn request_immediate_replan(&mut self) {
        self.immediate_replan.request(&self.ctx.events);
    }

    pub(super) fn consume_immediate_replan(&mut self) {
        self.immediate_replan.consume();
    }
}
