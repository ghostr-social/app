use super::{request_admitted, WarpPlannerInput};
use crate::adaptive::ActionNode;

pub(super) fn recovery_available(input: &WarpPlannerInput<'_>, frontier: &[ActionNode]) -> bool {
    input.playback_emergency()
        && frontier.iter().any(|node| {
            node.post == input.snapshot.playback.current
                && node.forecast.ready_playback_ms > 0
                && request_admitted(input, node)
        })
}
