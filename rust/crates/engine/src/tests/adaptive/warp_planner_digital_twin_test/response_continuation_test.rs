use super::*;
use crate::ActionId;

#[test]
fn request_free_continuation_uses_its_physical_body_forecast() {
    let action = ActionNode::new(
        1,
        PostId::new("video"),
        ActionKind::Promote {
            active: ActionId::new(17),
            maximum_bytes: 200_000,
        },
        ActionValue::from_net_micros(2_000_000),
    )
    .with_resources(ResourceCost::new(200_000, 200_000, 0, 0))
    .with_forecast(ActionForecast::new(
        CompletionTimes::new(508, 2_034, 8_138, 11_190),
        10_000,
        1_000,
    ));
    let state = TwinState::new(0, 4_000_000, 60_000, 0);
    let result = DigitalTwin::new(TwinConfig::new(64, 9_500)).evaluate(
        &state,
        &[action],
        TwinEpochs::new(1, 1, 1),
    );

    assert!(result.p99_visible_delay_ms <= 8_138);
}
