use crate::playback::EstimateConfidence;
use crate::rendition::QualitySelectionPolicy;
use crate::tests::rendition_support::{id, ladder, network, playing_input};

#[test]
fn separate_upgrade_and_downgrade_thresholds_prevent_quality_oscillation() {
    let policy = QualitySelectionPolicy::default();
    let boundary = network(7_400_000, 0, EstimateConfidence::High);
    let abundant = network(9_000_000, 0, EstimateConfidence::High);
    let constrained = network(6_400_000, 0, EstimateConfidence::High);

    let before_upgrade = policy.select(
        &ladder(),
        playing_input(boundary, Some("medium"), 20, 1_000),
    );
    let upgrade = policy.select(
        &ladder(),
        playing_input(abundant, Some("medium"), 20, 1_000),
    );
    let after_upgrade = policy.select(&ladder(), playing_input(boundary, Some("high"), 20, 1_000));
    let downgrade = policy.select(
        &ladder(),
        playing_input(constrained, Some("high"), 20, 1_000),
    );

    assert_eq!(before_upgrade.selected().id(), &id("medium"));
    assert_eq!(upgrade.selected().id(), &id("high"));
    assert_eq!(after_upgrade.selected().id(), &id("high"));
    assert_eq!(downgrade.selected().id(), &id("medium"));
}
