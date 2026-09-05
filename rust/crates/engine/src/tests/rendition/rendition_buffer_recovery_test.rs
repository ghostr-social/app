use crate::playback::EstimateConfidence;
use crate::rendition::QualitySelectionPolicy;
use crate::tests::rendition_support::{id, ladder, network, playing_input};

#[test]
fn a_buffer_below_steady_target_recovers_on_a_lower_rendition() {
    let policy = QualitySelectionPolicy::default();
    let network = network(7_500_000, 100_000, EstimateConfidence::High);

    let decision = policy.select(&ladder(), &playing_input(network, Some("high"), 3, 1_000));

    assert_eq!(decision.selected().id(), &id("medium"));
}
