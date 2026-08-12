use crate::playback::{EstimateConfidence, PlaybackPhase};
use crate::rendition::{QualityChange, QualitySelectionPolicy};
use crate::tests::rendition_support::{ladder, network, phase_input};

#[test]
fn inactive_playback_holds_quality_instead_of_reacting_to_network_noise() {
    let policy = QualitySelectionPolicy::default();
    let poor = network(1_500_000, 500_000, EstimateConfidence::Low);

    for phase in [
        PlaybackPhase::Paused,
        PlaybackPhase::Ended,
        PlaybackPhase::Inactive,
    ] {
        let decision = policy.select(&ladder(), phase_input(poor, Some("high"), 20, phase));
        assert_eq!(decision.selected().id().as_str(), "high");
        assert_eq!(decision.change(), QualityChange::Maintained);
    }
}
