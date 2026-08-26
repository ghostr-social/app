use crate::adaptive::{AdaptivePlayabilityPolicy, PlayerPreparation};
use crate::tests::adaptive_support::snapshot;

#[test]
fn only_a_rendered_first_frame_satisfies_the_ready_reserve() {
    let cases = [
        (PlayerPreparation::Unverified, 0),
        (PlayerPreparation::Initializing, 0),
        (PlayerPreparation::PluginReady, 0),
        (PlayerPreparation::Failed, 0),
        (PlayerPreparation::FirstFrameRendered, 1),
    ];

    for (preparation, expected_ready) in cases {
        let mut input = snapshot(2, 20_000_000, 20_000, 0);
        let startup = input.candidates[1]
            .startup
            .as_ref()
            .expect("valid test fixture")
            .ranges()[0];
        input.candidates[1].present = vec![startup];
        input.candidates[1].player_preparation = preparation;

        let plan = AdaptivePlayabilityPolicy.plan(&input);

        assert_eq!(plan.ready_reserve.ready, expected_ready);
        assert_eq!(plan.ready_reserve.ready + plan.ready_reserve.structural, 1,);
    }
}
