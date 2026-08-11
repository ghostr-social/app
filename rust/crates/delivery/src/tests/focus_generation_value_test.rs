use crate::delivery_events::FocusGeneration;

#[test]
fn generation_values_distinguish_ordered_focus_from_compatibility_calls() {
    assert!(FocusGeneration::try_new(0).is_none());
    assert_eq!(FocusGeneration::try_new(7).unwrap().value(), Some(7));
    assert_eq!(FocusGeneration::compatibility().value(), None);
}
