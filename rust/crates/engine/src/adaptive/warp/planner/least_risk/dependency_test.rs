use super::choose;
use crate::adaptive::{ActionForecast, ActionKind, ActionNode, ActionValue, CompletionTimes};
use crate::{ByteRange, PostId};

#[test]
fn degraded_choice_never_skips_an_unfinished_dependency() {
    let root = node(1, ActionKind::Head, 10, 0).requiring(&[]);
    let dependent = node(2, range(2), 1, 1).requiring(&[1]);

    assert_eq!(
        choose(&[root, dependent], &[])
            .action
            .expect("valid test fixture")
            .id,
        1
    );
}

#[test]
fn degraded_choice_advances_readiness_before_zero_gain_head() {
    let head = node(1, ActionKind::Head, 1, 0);
    let whole = node(
        2,
        ActionKind::FetchWhole {
            maximum_bytes: 285_652,
        },
        10,
        6_000,
    );

    assert_eq!(
        choose(&[head, whole], &[])
            .action
            .expect("valid test fixture")
            .id,
        2
    );
}

#[test]
fn preferred_terminal_selects_its_unfinished_dependency_root() {
    let unrelated = node(1, range(1), 1, 2_000);
    let root = node(2, ActionKind::Head, 10, 0);
    let terminal = node(3, range(3), 20, 2_000).requiring(&[2]);

    assert_eq!(
        choose(&[unrelated, root, terminal], &[3])
            .action
            .expect("preferred dependency root")
            .id,
        2
    );
}

fn node(id: u16, kind: ActionKind, p99_ms: u64, ready_ms: u64) -> ActionNode {
    ActionNode::new(
        id,
        PostId::new("post"),
        kind,
        ActionValue::from_net_micros(1),
    )
    .with_forecast(ActionForecast::new(
        CompletionTimes::new(1, 1, p99_ms, p99_ms),
        10_000,
        ready_ms,
    ))
}

fn range(id: u16) -> ActionKind {
    ActionKind::FetchRange(ByteRange::new(u64::from(id), u64::from(id) + 1))
}
