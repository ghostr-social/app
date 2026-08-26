use crate::debug::network::NetworkProfile;
use crate::delivery_events::{command_channel, DeliveryCommand};

#[test]
fn network_profile_coalescing_preserves_the_newest_generation_and_profile() {
    let (handle, mut receiver) = command_channel();
    let _ = handle.update_network_profile(profile(100));
    let generation = handle.update_network_profile(profile(200)).expect("valid test fixture");

    let Some(DeliveryCommand::NetworkProfile {
        generation: actual,
        profile,
    }) = receiver.try_control()
    else {
        panic!("missing network profile");
    };
    assert_eq!(actual, generation);
    assert_eq!(profile.latency_ms, 200);
    assert!(receiver.try_control().is_none());
}

fn profile(latency_ms: u64) -> NetworkProfile {
    NetworkProfile {
        latency_ms,
        ..NetworkProfile::default()
    }
}
