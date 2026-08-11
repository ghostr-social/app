use crate::playback::EstimateConfidence;
use crate::rendition::QualitySelectionPolicy;
use crate::tests::rendition_support::{ladder, network, playing_input};

#[test]
fn accelerated_playback_reduces_the_safely_sustainable_rendition() {
    let policy = QualitySelectionPolicy::default();
    let network = network(10_000_000, 0, EstimateConfidence::High);

    let normal = policy.select(&ladder(), playing_input(network, None, 20, 1_000));
    let doubled = policy.select(&ladder(), playing_input(network, None, 20, 2_000));

    assert_eq!(normal.selected().id().as_str(), "high");
    assert_eq!(doubled.selected().id().as_str(), "medium");
}
