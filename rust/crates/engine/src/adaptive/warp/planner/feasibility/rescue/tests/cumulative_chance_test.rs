use super::{exact_limits, node, select_path};
use crate::adaptive::{ActionForecast, CompletionTimes, ResourceCost};

#[test]
fn rescue_chance_constraints_cover_the_complete_path() {
    let mut first = node(1, ResourceCost::new(20, 20, 0, 1), 0, &[]);
    let mut terminal = node(2, ResourceCost::new(20, 20, 0, 1), 1_000, &[1]);
    first.forecast = forecast(1_500, 2_000, 0);
    terminal.forecast = forecast(1_500, 2_000, 1_000);

    assert!(select_path(&[first, terminal], exact_limits()).is_none());
}

fn forecast(p95_ms: u64, p99_ms: u64, ready_ms: u64) -> ActionForecast {
    ActionForecast::new(
        CompletionTimes::new(10, p95_ms, p99_ms, p99_ms),
        10_000,
        ready_ms,
    )
}
