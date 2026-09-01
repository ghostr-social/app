use crate::debug::network::NetworkProfile;
use crate::delivery_events::{
    command_channel, DeliveryCommand, DeliveryFocus, DeliveryNetworkStatus,
};
use ghostr_engine::origin_model::NetworkClass;
use ghostr_engine::DataUsageLevel;

#[test]
fn replaceable_controls_coalesce_to_the_latest_pending_intent() {
    let (handle, mut receiver) = command_channel();
    handle.update_focus(focus(1));
    handle.update_focus(focus(2));
    handle.set_data_usage(DataUsageLevel::Conservative);
    handle.set_data_usage(DataUsageLevel::Aggressive);

    let commands: Vec<_> = core::iter::from_fn(|| receiver.try_control()).collect();

    assert_eq!(commands.len(), 2);
    assert!(commands.iter().any(latest_focus));
    assert!(commands.iter().any(latest_config));
}

#[test]
fn focus_batch_preserves_earlier_controls_and_leaves_the_suffix() {
    let (handle, mut receiver) = command_channel();
    handle.set_data_usage(DataUsageLevel::Conservative);
    handle.update_focus(focus(1));
    let generation = handle
        .update_network_profile(NetworkProfile::default())
        .expect("valid test fixture");

    let commands = receiver
        .try_controls_through_focus()
        .expect("valid test fixture");

    assert!(matches!(
        commands.as_slice(),
        [
            DeliveryCommand::Config(DataUsageLevel::Conservative),
            DeliveryCommand::Focus(_)
        ]
    ));
    assert!(matches!(
        receiver.try_control(),
        Some(DeliveryCommand::NetworkProfile {
            generation: actual,
            ..
        }) if actual == generation
    ));
}

#[test]
fn network_status_coalescing_keeps_only_the_freshest_generation() {
    let (handle, mut receiver) = command_channel();
    assert!(handle.update_network_status(status(NetworkClass::Wifi, 5)));
    assert!(handle.update_network_status(status(NetworkClass::Cellular, 4)));
    assert!(handle.update_network_status(status(NetworkClass::Cellular, 5)));

    let Some(DeliveryCommand::NetworkStatus(latest)) = receiver.try_control() else {
        panic!("missing network status");
    };
    assert_eq!(latest.network_class(), NetworkClass::Wifi);
    assert_eq!(latest.generation(), 5);
    assert!(handle.update_network_status(status(NetworkClass::Wired, 6)));
    let Some(DeliveryCommand::NetworkStatus(latest)) = receiver.try_control() else {
        panic!("missing replacement network status");
    };
    assert_eq!(latest.network_class(), NetworkClass::Wired);
    assert_eq!(latest.generation(), 6);
    assert!(receiver.try_control().is_none());
}

fn focus(watch_ms: u64) -> DeliveryFocus {
    DeliveryFocus::compatibility(Vec::new(), 0, watch_ms)
}

fn status(network: NetworkClass, generation: u64) -> DeliveryNetworkStatus {
    DeliveryNetworkStatus::new(network, generation)
}

fn latest_focus(command: &DeliveryCommand) -> bool {
    matches!(command, DeliveryCommand::Focus(focus) if focus.watch_ms == 2)
}

fn latest_config(command: &DeliveryCommand) -> bool {
    matches!(command, DeliveryCommand::Config(DataUsageLevel::Aggressive))
}
