use super::{SegmentedDelivery, SegmentedFocusItem, Target};
use crate::delivery_events::DeliveryFocus;
use ghostr_engine::DeliveryKind;

impl SegmentedDelivery {
    pub(super) fn equivalent_work(
        &self,
        tracked: &[SegmentedFocusItem],
        targets: &[Target],
        delivery: Option<DeliveryKind>,
    ) -> bool {
        self.current_delivery == delivery
            && same_sources(&self.tracked, tracked)
            && self.targets == targets
    }

    pub(super) fn refresh_tracked_identity(
        &mut self,
        focus: &DeliveryFocus,
        tracked: Vec<SegmentedFocusItem>,
        targets: &[Target],
    ) {
        if self.tracked == tracked {
            return;
        }
        let generation = self.generation(focus);
        let preserved = self.reconcile_work(targets);
        let protected = targets.iter().map(|target| target.post.clone()).collect();
        self.cache
            .reconcile_focus_window(generation, tracked.clone(), &protected, &preserved);
        self.tracked = tracked;
    }
}

fn same_sources(left: &[SegmentedFocusItem], right: &[SegmentedFocusItem]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| left.post() == right.post() && left.sources() == right.sources())
}
