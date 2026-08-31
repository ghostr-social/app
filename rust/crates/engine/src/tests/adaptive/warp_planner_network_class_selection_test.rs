use super::network_class_crossover_support::fixture;
use crate::adaptive::{WarpPlanner, WarpPlannerConfig, WarpPlannerInput};
use crate::origin_model::NetworkClass;

#[test]
fn global_network_class_flips_the_selected_progressive_source() {
    assert_eq!(selected_post(NetworkClass::Wifi), "p0");
    assert_eq!(selected_post(NetworkClass::Cellular), "p1");
}

fn selected_post(network_class: NetworkClass) -> String {
    let fixture = fixture(network_class);
    WarpPlanner::new(WarpPlannerConfig::default().with_legacy_reserve_progress_for_test())
        .plan(WarpPlannerInput::new(
            &fixture.snapshot,
            &fixture.base,
            &fixture.origins,
            &fixture.context,
        ))
        .selected
        .expect("selected plan")
        .node
        .post
        .as_str()
        .to_owned()
}
