use crate::adaptive::{
    ActionForecast, ActionKind, ActionNode, ActionValue, CompletionTimes, ResourceCost,
};
use crate::{ByteRange, PostId};

pub(super) fn action(id: u16, bytes: u64, p95: u64, gain: u64) -> ActionNode {
    ActionNode::new(
        id,
        PostId::new("video"),
        ActionKind::FetchRange(ByteRange::new(0, bytes)),
        ActionValue::from_net_micros(1_000_000),
    )
    .with_resources(ResourceCost::new(bytes, bytes, 0, 1))
    .with_forecast(ActionForecast::new(
        CompletionTimes::new(p95 / 2, p95, p95 * 2, p95 * 3),
        9_000,
        gain,
    ))
}
