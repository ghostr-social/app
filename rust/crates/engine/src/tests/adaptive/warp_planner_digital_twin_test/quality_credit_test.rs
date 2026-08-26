use crate::adaptive::{
    ActionForecast, ActionKind, ActionNode, ActionValue, CompletionTimes, DigitalTwin, TwinConfig,
    TwinEpochs, TwinState,
};
use crate::{ByteRange, PostId};

#[test]
fn rendition_transition_is_credited_once_per_post() {
    let mut twin = DigitalTwin::new(TwinConfig::new(1, 9_500));
    let state = TwinState::new(0, 1, 0, 1);
    let epochs = TwinEpochs::new(1, 1, 1);
    let first = action(1, "post-a", 0);
    let second = action(2, "post-a", 1);
    let other = action(2, "post-b", 1);

    let once = twin.evaluate(&state, core::slice::from_ref(&first), epochs);
    let repeated = twin.evaluate(&state, &[first.clone(), second], epochs);
    let independent = twin.evaluate(&state, &[first, other], epochs);

    assert_eq!(once.expected_score_micros, 50_000);
    assert_eq!(repeated.expected_score_micros, 50_000);
    assert_eq!(independent.expected_score_micros, 100_000);
}

fn action(id: u16, post: &str, start: u64) -> ActionNode {
    ActionNode::new(
        id,
        PostId::new(post),
        ActionKind::FetchRange(ByteRange::new(start, start + 1)),
        ActionValue::default(),
    )
    .with_forecast(
        ActionForecast::new(CompletionTimes::new(0, 0, 0, 0), 10_000, 1).with_quality(50_000),
    )
}
