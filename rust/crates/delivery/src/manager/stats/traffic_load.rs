use crate::manager::traffic::OverallTrafficWindow;
use log::trace;

#[derive(Default)]
pub(super) struct TrafficLoad {
    latest: Option<OverallTrafficWindow>,
}

impl TrafficLoad {
    pub(super) fn observe(&mut self, window: OverallTrafficWindow) {
        trace!(
            "traffic window: bytes={}, elapsed={:?}, rate={}, peak={}, at={}, ttfb={:?}",
            window.bytes(),
            window.elapsed(),
            window.bytes_per_second(),
            window.peak_active_transfers(),
            window.observed_at_ms(),
            window.latest_ttfb(),
        );
        self.latest = Some(window);
    }

    pub(super) fn bytes_per_second_at(&self, observed_at_ms: u64) -> u64 {
        self.latest
            .filter(|window| window.fresh_at(observed_at_ms))
            .map_or(0, OverallTrafficWindow::measured_bytes_per_second)
    }
}
