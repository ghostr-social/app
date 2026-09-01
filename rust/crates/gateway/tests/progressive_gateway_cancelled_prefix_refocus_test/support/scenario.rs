use crate::gateway_fixture::{
    progressive_delivery::ProgressiveDeliveryHarness, progressive_fixture_bytes,
};
use crate::support::{
    focus_and_wait, focus_trimmed_and_wait, held_prefix_after_tail, pending_transfer_sequence,
    roster, seed_ready_ranges, wait_closed, wait_for_tail, wait_for_zero_byte_cancellation,
    ControlledOrigin,
};
use ghostr_delivery::delivery_events::FocusItem;

pub struct CancelledPrefixScenario {
    pub(super) origin: ControlledOrigin,
    pub(super) harness: ProgressiveDeliveryHarness,
    pub(super) items: Vec<FocusItem>,
    pub(super) bytes: Vec<u8>,
}

impl CancelledPrefixScenario {
    pub async fn start() -> Self {
        let bytes = progressive_fixture_bytes();
        let origin = ControlledOrigin::serve(bytes.clone()).await;
        let harness = ProgressiveDeliveryHarness::start("ghostr-gateway-cancel-refocus");
        let items = roster(&origin);
        seed_ready_ranges(&harness, &items, &bytes).await;
        initial_focus(&harness, &items).await;
        Self {
            origin,
            harness,
            items,
            bytes,
        }
    }

    pub async fn cancel_speculative_prefix(&mut self) {
        let prefix = held_prefix_after_tail(&mut self.origin).await;
        wait_for_tail(&self.harness).await;
        let sequence = pending_transfer_sequence(&self.harness.delivery.handle);
        reverse_focus(&self.harness, &self.items).await;
        wait_closed(&prefix).await;
        wait_for_zero_byte_cancellation(&self.harness.delivery.handle, sequence).await;
    }

    pub async fn advance_before_target(&self) {
        for (generation, original) in (8..=12).zip(1..=5) {
            focus_trimmed_and_wait(&self.harness, &self.items, original, generation).await;
        }
    }
}

async fn initial_focus(harness: &ProgressiveDeliveryHarness, items: &[FocusItem]) {
    for (generation, current) in (1..=4).zip(0..=3) {
        focus_and_wait(harness, items, current, generation).await;
    }
}

async fn reverse_focus(harness: &ProgressiveDeliveryHarness, items: &[FocusItem]) {
    for (generation, current) in [(5, 2), (6, 1), (7, 0)] {
        focus_and_wait(harness, items, current, generation).await;
    }
}
