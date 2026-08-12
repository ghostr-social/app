use super::candidate_catalog_fixture::{binding, candidate};
use crate::manager::state::DeliveryState;
use ghostr_engine::{DataUsageLevel, EngineParams, PostId};

#[test]
fn unchanged_bindings_are_not_requeued_for_installation() {
    let mut state = state();
    state.apply_candidate(candidate("same", 0));
    let binding = binding(&mut state, "same");

    state.queue_representation(binding.clone());
    state.queue_representation(binding);

    assert_eq!(state.take_representation_bindings().len(), 1);
}

#[test]
fn repeated_candidate_metadata_does_not_queue_the_same_binding() {
    let mut state = state();
    state.apply_candidate(candidate("same", 0));
    state.take_representation_bindings();

    state.apply_candidate(candidate("same", 1));

    assert!(state.take_representation_bindings().is_empty());
}

#[test]
fn changed_candidate_metadata_reports_one_representation_change() {
    let mut state = state();
    state.apply_candidate(candidate("same", 0));
    state.take_representation_bindings();

    let mut changed = candidate("same", 1);
    changed.meta.size_bytes = Some(32);
    state.apply_candidate(changed);

    assert_eq!(state.take_changed_representations(), [PostId::new("same")]);
    assert!(state.take_changed_representations().is_empty());
}

fn state() -> DeliveryState {
    DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced)
}
