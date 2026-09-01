use crate::delivery_events::{
    command_channel, DeliveryCommand, DeliveryFocus, FocusAdmission, FocusGeneration,
    FocusTransition,
};

#[test]
fn stale_focus_is_rejected_after_the_newer_intent_is_consumed() {
    let (handle, mut receiver) = command_channel();

    assert_eq!(handle.update_focus(focus(2)), FocusAdmission::Accepted);
    let command = receiver
        .try_control()
        .expect("newest focus remains pending");
    assert!(matches!(
        command,
        DeliveryCommand::Focus(focus) if focus.generation == generation(2)
    ));

    assert_eq!(handle.update_focus(focus(1)), FocusAdmission::Stale);
    assert!(receiver.try_control().is_none());
}

#[test]
fn clearing_pending_work_does_not_reopen_an_old_generation() {
    let (handle, receiver) = command_channel();
    assert_eq!(handle.update_focus(focus(2)), FocusAdmission::Accepted);

    receiver.discard_pending();

    assert_eq!(handle.update_focus(focus(1)), FocusAdmission::Stale);
}

fn focus(value: u64) -> DeliveryFocus {
    DeliveryFocus {
        items: Vec::new(),
        previews: Vec::new(),
        current_index: 0,
        watch_ms: 0,
        generation: generation(value),
        transition: FocusTransition::UserNavigation,
        rescue: None,
    }
}

fn generation(value: u64) -> FocusGeneration {
    FocusGeneration::try_new(value).expect("positive generation")
}
