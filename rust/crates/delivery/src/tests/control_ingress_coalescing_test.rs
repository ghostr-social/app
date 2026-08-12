use crate::delivery_events::{command_channel, DeliveryCommand, DeliveryFocus};
use ghostr_engine::DataUsageLevel;

#[test]
fn replaceable_controls_coalesce_to_the_latest_pending_intent() {
    let (handle, mut receiver) = command_channel();
    handle.update_focus(focus(1));
    handle.update_focus(focus(2));
    handle.set_data_usage(DataUsageLevel::Conservative);
    handle.set_data_usage(DataUsageLevel::Aggressive);

    let commands: Vec<_> = std::iter::from_fn(|| receiver.try_control()).collect();

    assert_eq!(commands.len(), 2);
    assert!(commands.iter().any(latest_focus));
    assert!(commands.iter().any(latest_config));
}

fn focus(watch_ms: u64) -> DeliveryFocus {
    DeliveryFocus::compatibility(Vec::new(), 0, watch_ms)
}

fn latest_focus(command: &DeliveryCommand) -> bool {
    matches!(command, DeliveryCommand::Focus(focus) if focus.watch_ms == 2)
}

fn latest_config(command: &DeliveryCommand) -> bool {
    matches!(command, DeliveryCommand::Config(DataUsageLevel::Aggressive))
}
