use crate::engine::scoring::position_weight;

#[test]
fn the_current_post_carries_full_weight() {
    assert_eq!(position_weight(0), 1.0);
}

#[test]
fn weight_decays_with_every_step_ahead() {
    assert!(position_weight(1) < position_weight(0));
    assert!(position_weight(2) < position_weight(1));
    assert!(position_weight(6) < position_weight(3));
}

#[test]
fn weight_decays_with_every_step_behind() {
    assert!(position_weight(-2) < position_weight(-1));
    assert!(position_weight(-3) < position_weight(-2));
}

#[test]
fn behind_posts_are_heavily_discounted_against_ahead_posts() {
    assert!(position_weight(-1) < position_weight(1));
    assert!(position_weight(-1) < position_weight(6));
}

#[test]
fn weights_stay_positive_and_finite_at_extreme_distances() {
    for distance in [-1_000, -64, 64, 1_000] {
        let weight = position_weight(distance);
        assert!(weight.is_finite() && weight > 0.0, "distance {distance}");
    }
}
