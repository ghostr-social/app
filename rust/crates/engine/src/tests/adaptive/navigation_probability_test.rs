use crate::adaptive::{FeedOffset, NavigationSnapshot};

#[test]
fn directional_navigation_evidence_moves_probability_across_the_current_item() {
    let neutral = NavigationSnapshot {
        forward_swipes_per_minute: 0,
        backward_swipes_per_minute: 0,
    };
    let backward = NavigationSnapshot {
        forward_swipes_per_minute: 0,
        backward_swipes_per_minute: 30,
    };

    assert!(
        neutral.view_probability(FeedOffset::new(1)).value()
            > neutral.view_probability(FeedOffset::new(-1)).value()
    );
    assert!(
        backward.view_probability(FeedOffset::new(-1)).value()
            > backward.view_probability(FeedOffset::new(1)).value()
    );
    assert_eq!(backward.view_probability(FeedOffset::new(0)).value(), 1.0);
}
