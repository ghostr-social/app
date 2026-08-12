use crate::playback::{EstimateConfidence, PlaybackPhase};
use crate::rendition::{QualityChange, QualitySelectionPolicy};
use crate::tests::rendition_support::{ladder, network, phase_input};

#[test]
fn startup_uses_extra_capacity_margin_before_committing_to_quality() {
    let policy = QualitySelectionPolicy::default();
    let network = network(11_000_000, 200_000, EstimateConfidence::High);

    let decision = policy.select(
        &ladder(),
        phase_input(network, None, 0, PlaybackPhase::Starting),
    );

    assert_eq!(decision.selected().id().as_str(), "medium");
    assert_eq!(decision.change(), QualityChange::Initial);
}
