use crate::manager::traffic::{OverallTrafficWindow, SAMPLE_INTERVAL};
use std::time::Duration;

#[test]
fn partial_interval_bytes_do_not_become_a_spurious_load_spike() {
    let window = OverallTrafficWindow::new(2_000, Duration::from_millis(1), 1, 10, None);

    assert_eq!(
        window.measured_bytes_per_second(),
        2_000 * 1_000 / SAMPLE_INTERVAL.as_millis() as u64
    );
}
