use super::choose;
use crate::adaptive::{ActionForecast, ActionKind, ActionNode, ActionValue, CompletionTimes};
use crate::{ByteRange, PostId};

#[test]
fn degraded_choice_never_skips_an_unfinished_dependency() {
    let root = node(1, 10, &[]);
    let dependent = node(2, 1, &[1]);

    assert_eq!(choose(&[root, dependent]).action.unwrap().id, 1);
}

fn node(id: u16, p99_ms: u64, requires: &[u16]) -> ActionNode {
    ActionNode::new(
        id,
        PostId::new("post"),
        ActionKind::FetchRange(ByteRange::new(u64::from(id), u64::from(id) + 1)),
        ActionValue::from_net_micros(1),
    )
    .with_forecast(ActionForecast::new(
        CompletionTimes::new(1, 1, p99_ms, p99_ms),
        10_000,
        1,
    ))
    .requiring(requires)
}
