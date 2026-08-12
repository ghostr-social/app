use crate::playback::{EstimateConfidence, PlaybackPhase};
use crate::rendition::QualitySelectionPolicy;
use crate::tests::rendition_support::{id, ladder, network, phase_input, playing_input};

#[test]
fn critical_buffer_risk_downgrades_immediately_to_create_refill_margin() {
    let policy = QualitySelectionPolicy::default();
    let network = network(8_000_000, 200_000, EstimateConfidence::High);

    let comfortable = policy.select(&ladder(), playing_input(network, Some("high"), 20, 1_000));
    let nearly_empty = policy.select(&ladder(), playing_input(network, Some("high"), 1, 1_000));
    let stalled = policy.select(
        &ladder(),
        phase_input(network, Some("high"), 1, PlaybackPhase::NetworkStalled),
    );

    assert_eq!(comfortable.selected().id(), &id("high"));
    for risky in [nearly_empty, stalled] {
        assert_eq!(risky.selected().id(), &id("medium"));
    }
}
