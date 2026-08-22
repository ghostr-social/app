use crate::adaptive::{
    ActionForecast, ActionKind, ActionNode, ActionValue, CompletionTimes, DigitalTwin,
    ResourceCost, TwinConfig, TwinEpochs, TwinState,
};
use crate::PostId;

#[test]
fn same_shape_different_origin_is_consistent_between_cached_and_fresh_twin() {
    let state = TwinState::new(0, 8_000_000, 25, 2);
    let epochs = TwinEpochs::new(7, 11, 13);
    let first = action("post-a", "https://first.example/media");
    let second = action("post-b", "https://second.example/media");
    let mut reused = DigitalTwin::new(TwinConfig::new(64, 9_500));

    reused.evaluate(&state, &[first], epochs);
    let cached = reused.evaluate(&state, std::slice::from_ref(&second), epochs);
    let fresh = DigitalTwin::new(TwinConfig::new(64, 9_500)).evaluate(&state, &[second], epochs);

    assert_eq!(cached, fresh);
}

fn action(post: &str, source: &str) -> ActionNode {
    ActionNode::new(
        1,
        PostId::new(post),
        ActionKind::FetchWhole {
            maximum_bytes: 512_000,
        },
        ActionValue::from_net_micros(1_000_000),
    )
    .with_origin(source)
    .with_resources(ResourceCost::new(512_000, 64_000, 0, 1))
    .with_forecast(ActionForecast::new(
        CompletionTimes::new(100, 400, 900, 1_200),
        5_000,
        2_000,
    ))
}
