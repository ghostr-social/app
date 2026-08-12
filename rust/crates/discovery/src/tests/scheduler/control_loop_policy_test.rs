//! Demand-transition policy table (plan §5.4 unified control loop):
//! expansion widens the active feed's querying; hold stays quiet.

use crate::scheduler::control::{discovery_action, DiscoveryAction, FeedQueryState};
use ghostr_engine::adaptive::DiscoveryDemand;

fn feed() -> FeedQueryState {
    FeedQueryState {
        open: true,
        ..FeedQueryState::default()
    }
}

#[test]
fn demand_policy_table() {
    let ready = FeedQueryState {
        has_cursor: true,
        loaded: true,
        ..feed()
    };
    let cases = [
        (DiscoveryDemand::Hold, ready, DiscoveryAction::Idle),
        (
            DiscoveryDemand::Hold,
            FeedQueryState {
                loaded: true,
                ..feed()
            },
            DiscoveryAction::Idle,
        ),
        (
            DiscoveryDemand::Expand,
            FeedQueryState::default(),
            DiscoveryAction::Idle,
        ),
        (
            DiscoveryDemand::Expand,
            FeedQueryState {
                busy: true,
                ..ready
            },
            DiscoveryAction::Idle,
        ),
        (
            DiscoveryDemand::Expand,
            ready,
            DiscoveryAction::PrefetchNextPage,
        ),
        (DiscoveryDemand::Expand, feed(), DiscoveryAction::Idle),
        (
            DiscoveryDemand::Expand,
            FeedQueryState {
                loaded: true,
                ..feed()
            },
            DiscoveryAction::WidenActiveQuery,
        ),
        (
            DiscoveryDemand::Expand,
            FeedQueryState {
                loaded: true,
                widened: true,
                ..feed()
            },
            DiscoveryAction::Idle,
        ),
    ];

    for (demand, state, expected) in cases {
        assert_eq!(
            discovery_action(demand, state),
            expected,
            "{demand:?} over {state:?}"
        );
    }
}
