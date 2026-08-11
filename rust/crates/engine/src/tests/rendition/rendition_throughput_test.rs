use crate::playback::EstimateConfidence;
use crate::rendition::{QualityChange, QualitySelectionPolicy};
use crate::tests::rendition_support::{ladder, network, playing_input};

#[test]
fn initial_quality_is_the_highest_rendition_with_sustainable_headroom() {
    let policy = QualitySelectionPolicy::default();
    let network = network(11_000_000, 200_000, EstimateConfidence::High);

    let decision = policy.select(&ladder(), playing_input(network, None, 20, 1_000));

    assert_eq!(decision.selected().id().as_str(), "high");
    assert_eq!(decision.selected().bitrate_bits_per_second(), 6_000_000);
    assert_eq!(decision.change(), QualityChange::Initial);
}
