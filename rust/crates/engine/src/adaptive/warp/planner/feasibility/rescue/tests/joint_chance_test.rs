use super::{exact_limits, node, select_path_at};
use crate::adaptive::{ActionForecast, CompletionTimes, ResourceCost};

#[test]
fn path_chance_spends_transport_and_timing_failure_together() {
    let mut first = node(1, ResourceCost::new(20, 20, 0, 1), 0, &[]);
    let mut terminal = node(2, ResourceCost::new(20, 20, 0, 1), 1_000, &[1]);
    first.forecast = forecast(0);
    terminal.forecast = forecast(1_000);

    assert!(select_path_at(&[first, terminal], exact_limits(), 8_800).is_none());
}

fn forecast(ready_ms: u64) -> ActionForecast {
    ActionForecast::new(
        CompletionTimes::new(10, 1_250, 2_000, 2_000),
        9_850,
        ready_ms,
    )
}
