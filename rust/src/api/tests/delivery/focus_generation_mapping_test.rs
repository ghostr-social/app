use crate::api::delivery::focus_mapping::delivery_focus;
use crate::api::focus_control::FfiFocusTransition;

#[test]
fn focus_generation_must_be_positive() {
    let error = delivery_focus(&[], 0, 0, 0, FfiFocusTransition::UserNavigation, None)
        .expect_err("zero is not a generation");

    assert!(error.to_string().contains("positive"));
}
