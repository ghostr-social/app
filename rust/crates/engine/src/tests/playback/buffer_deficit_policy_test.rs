use crate::playback::{
    AdaptiveBufferPolicy, EstimateConfidence, MediaConsumption, NetworkConditions,
};
use core::time::Duration;

#[test]
fn a_resource_cap_does_not_erase_the_predicted_buffer_requirement() {
    let target = AdaptiveBufferPolicy::default().target(
        NetworkConditions::new(0, 0, Duration::ZERO, EstimateConfidence::High),
        MediaConsumption::new(8_000_000, 2_000),
    );
    assert_eq!(target.required(), Duration::from_secs(40));
    assert_eq!(target.steady(), Duration::from_secs(30));
    assert!(!target.fits_retention_limit());
}

#[test]
fn the_buffer_horizon_ends_at_the_remaining_presentation_boundary() {
    let target = AdaptiveBufferPolicy::default().target_for(
        NetworkConditions::new(0, 0, Duration::ZERO, EstimateConfidence::High),
        MediaConsumption::new(8_000_000, 2_000),
        Duration::from_secs(1),
    );
    assert_eq!(target.required(), Duration::from_secs(1));
    assert!(target.fits_retention_limit());
}
