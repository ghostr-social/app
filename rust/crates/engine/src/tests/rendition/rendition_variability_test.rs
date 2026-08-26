use crate::playback::EstimateConfidence;
use crate::rendition::QualitySelectionPolicy;
use crate::tests::rendition_support::{id, ladder, network, playing_input};

#[test]
fn variability_is_discounted_more_when_throughput_confidence_is_low() {
    let policy = QualitySelectionPolicy::default();
    let stable = network(11_000_000, 200_000, EstimateConfidence::High);
    let uncertain = network(11_000_000, 2_500_000, EstimateConfidence::Low);

    let stable = policy.select(&ladder(), &playing_input(stable, None, 20, 1_000));
    let uncertain = policy.select(&ladder(), &playing_input(uncertain, None, 40, 1_000));

    assert_eq!(stable.selected().id(), &id("high"));
    assert_eq!(uncertain.selected().id(), &id("medium"));
}
