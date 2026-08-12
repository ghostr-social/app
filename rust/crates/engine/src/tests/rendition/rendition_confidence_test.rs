use crate::playback::EstimateConfidence;
use crate::rendition::QualitySelectionPolicy;
use crate::tests::rendition_support::{id, ladder, network, playing_input};

#[test]
fn an_upgrade_waits_for_high_confidence_even_with_abundant_bandwidth() {
    let policy = QualitySelectionPolicy::default();
    let low = network(20_000_000, 0, EstimateConfidence::Low);
    let medium = network(20_000_000, 0, EstimateConfidence::Medium);
    let high = network(20_000_000, 0, EstimateConfidence::High);

    for evidence in [low, medium] {
        let decision = policy.select(
            &ladder(),
            playing_input(evidence, Some("medium"), 40, 1_000),
        );
        assert_eq!(decision.selected().id(), &id("medium"));
    }

    let upgraded = policy.select(&ladder(), playing_input(high, Some("medium"), 20, 1_000));
    assert_eq!(upgraded.selected().id(), &id("high"));
}
