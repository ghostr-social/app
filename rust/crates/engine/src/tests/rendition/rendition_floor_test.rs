use crate::playback::EstimateConfidence;
use crate::rendition::{DowngradeCause, QualityChange, QualitySelectionPolicy};
use crate::tests::rendition_support::{ladder, network, playing_input};

#[test]
fn the_lowest_rendition_is_the_safe_fallback_when_none_are_sustainable() {
    let policy = QualitySelectionPolicy::default();
    let unusable = network(500_000, 500_000, EstimateConfidence::Low);

    let decision = policy.select(
        &ladder(),
        playing_input(unusable, Some("medium"), 40, 1_000),
    );

    assert_eq!(decision.selected().id().as_str(), "low");
    assert_eq!(
        decision.change(),
        QualityChange::Downgraded(DowngradeCause::ThroughputRisk),
    );
}
