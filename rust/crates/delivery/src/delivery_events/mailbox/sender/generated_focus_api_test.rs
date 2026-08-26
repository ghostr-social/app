use super::{control, signal, DeliveryCommand, DeliveryFocus, MailboxSender};

impl MailboxSender {
    pub(in crate::delivery_events) fn send_generated_focus(
        &self,
        mut focus: DeliveryFocus,
    ) -> Option<u64> {
        if self.control_wake.is_closed() {
            return None;
        }
        let mut state = self.lock();
        let generation = state.focus_generations.allocate()?;
        focus.generation = generation;
        control::replace(&mut state.controls, DeliveryCommand::Focus(focus));
        drop(state);
        signal(&self.control_wake).then_some(generation.value()?)
    }
}
