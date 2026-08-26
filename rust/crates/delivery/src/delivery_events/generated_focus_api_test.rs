use super::{DeliveryFocus, DeliveryHandle};

impl DeliveryHandle {
    pub(crate) fn update_generated_focus(&self, focus: DeliveryFocus) -> Option<u64> {
        self.sender.send_generated_focus(focus)
    }
}
