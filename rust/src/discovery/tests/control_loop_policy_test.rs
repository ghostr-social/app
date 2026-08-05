//! Mode-transition policy table (plan §5.4 unified control loop):
//! hunger widens the active feed's querying, comfort stays quiet.

use crate::discovery::control_loop::{discovery_action, DiscoveryAction, FeedQueryState};
use crate::engine::inventory_controller::Mode;

fn feed() -> FeedQueryState {
    FeedQueryState {
        open: true,
        ..FeedQueryState::default()
    }
}

#[test]
fn mode_policy_table() {
    let ready = FeedQueryState {
        has_cursor: true,
        loaded: true,
        ..feed()
    };
    let cases = [
        (Mode::Comfort, ready, DiscoveryAction::Idle),
        (
            Mode::Comfort,
            FeedQueryState {
                loaded: true,
                ..feed()
            },
            DiscoveryAction::Idle,
        ),
        (
            Mode::Hunger,
            FeedQueryState::default(),
            DiscoveryAction::Idle,
        ),
        (
            Mode::Hunger,
            FeedQueryState {
                busy: true,
                ..ready
            },
            DiscoveryAction::Idle,
        ),
        (Mode::Hunger, ready, DiscoveryAction::PrefetchNextPage),
        (Mode::Hunger, feed(), DiscoveryAction::Idle),
        (
            Mode::Hunger,
            FeedQueryState {
                loaded: true,
                ..feed()
            },
            DiscoveryAction::WidenActiveQuery,
        ),
        (
            Mode::Hunger,
            FeedQueryState {
                loaded: true,
                widened: true,
                ..feed()
            },
            DiscoveryAction::Idle,
        ),
    ];

    for (mode, state, expected) in cases {
        assert_eq!(
            discovery_action(mode, state),
            expected,
            "{mode:?} over {state:?}"
        );
    }
}
