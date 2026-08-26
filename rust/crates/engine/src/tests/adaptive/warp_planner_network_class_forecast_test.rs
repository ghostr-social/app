use super::network_class_support::fixture;
use crate::adaptive::axiom_test_support::WarpActionGenerator;
use crate::adaptive::ActionKind;
use crate::origin_model::NetworkClass;

#[test]
fn same_url_progressive_forecast_uses_global_network_class() {
    let wifi_fixture = fixture(NetworkClass::Wifi);
    let cellular_fixture = fixture(NetworkClass::Cellular);
    let wifi = expected_range_ms(&generate(&wifi_fixture), "p0");
    let cellular = expected_range_ms(&generate(&cellular_fixture), "p0");
    assert!(wifi < cellular, "wifi={wifi}, cellular={cellular}");
}

fn generate(
    fixture: &super::network_class_support::NetworkClassFixture,
) -> crate::adaptive::GeneratedActions {
    WarpActionGenerator::generate(
        &fixture.snapshot,
        &fixture.base,
        &fixture.origins,
        &fixture.context,
    )
}

fn expected_range_ms(actions: &crate::adaptive::GeneratedActions, post: &str) -> u64 {
    actions
        .actions
        .iter()
        .find(|action| {
            action.node.post.as_str() == post
                && matches!(action.node.kind, ActionKind::FetchRange(_))
                && action.node.forecast.ready_playback_ms > 0
        })
        .expect("playable progressive range")
        .node
        .forecast
        .completion
        .expected_ms
}
