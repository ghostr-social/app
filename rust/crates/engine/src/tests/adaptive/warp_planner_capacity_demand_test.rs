use crate::adaptive::{
    AdaptivePlayabilityPolicy, InFlightAction, PlannerContext, ResourceFeedback,
    ResourceObservation, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::{ActionId, ByteRange};

#[path = "warp_planner_available_capacity_demand_test.rs"]
mod available_capacity_demand_test;

#[test]
fn one_more_slot_is_demanded_only_when_warp_would_use_it() {
    let demanded = decision(Scenario::viable());
    assert!(demanded.selected.is_none(), "demand must not dispatch work");
    assert_ne!(demanded.common_random_seed, 0);
    assert!(!demanded.reserve.degraded, "the viable path has a reserve");
    assert!(demanded.additional_request_slot_demanded);

    for blocked in [
        Scenario {
            per_authority: 1,
            ..Scenario::viable()
        },
        Scenario {
            ceiling: 1,
            ..Scenario::viable()
        },
        Scenario {
            active: 0,
            ..Scenario::viable()
        },
        Scenario {
            active: 2,
            ..Scenario::viable()
        },
        Scenario {
            priced_out: true,
            ..Scenario::viable()
        },
    ] {
        assert!(!decision(blocked).additional_request_slot_demanded);
    }
}

#[derive(Clone, Copy)]
struct Scenario {
    active: usize,
    capacity: usize,
    ceiling: usize,
    per_authority: usize,
    priced_out: bool,
}

impl Scenario {
    const fn viable() -> Self {
        Self {
            active: 1,
            capacity: 1,
            ceiling: 2,
            per_authority: 2,
            priced_out: false,
        }
    }
}

fn decision(scenario: Scenario) -> crate::adaptive::WarpPlanningDecision {
    let mut input = snapshot(1, 20_000_000, 8_000, 0);
    input.network.connection_capacity = scenario.capacity;
    input.network.connection_ceiling = scenario.ceiling;
    input.network.per_authority_request_limit = scenario.per_authority;
    for index in 0..scenario.active {
        let start = index as u64 * 64_000;
        input.candidates[0].in_flight.push(InFlightAction::range(
            ActionId::new(index as u64 + 1),
            ByteRange::new(start, start + 64_000),
            "https://origin.example/media",
            20_000,
            true,
        ));
    }
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let mut context = PlannerContext::explicitly_unavailable(&input);
    if scenario.priced_out {
        context = context.with_feedback(ResourceFeedback {
            revision: 1,
            actual: ResourceObservation::new(0, 0, 0, u64::MAX),
            target: ResourceObservation::new(1, 1, 1, 1),
            price_snapshot: None,
        });
    }
    WarpPlanner::default().plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ))
}
