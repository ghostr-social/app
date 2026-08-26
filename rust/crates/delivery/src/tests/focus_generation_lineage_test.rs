use crate::delivery_events::{command_channel, DeliveryCommand, DeliveryFocus, FocusItem};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn same_current_coalescing_covers_the_replaced_focus_generation() {
    let (handle, mut receiver) = command_channel();
    let first = handle.update_generated_focus(focus("a")).expect("valid test fixture");
    let second = handle.update_generated_focus(focus("a")).expect("valid test fixture");
    let DeliveryCommand::Focus(focus) = receiver.try_control().expect("valid test fixture") else {
        panic!("focus command");
    };

    assert_eq!(focus.generation.value(), Some(second));
    assert_eq!(focus.generation.covers_from_value(), Some(first));
}

#[test]
fn changed_current_does_not_cover_the_replaced_focus_generation() {
    let (handle, mut receiver) = command_channel();
    handle.update_generated_focus(focus("a")).expect("valid test fixture");
    let second = handle.update_generated_focus(focus("b")).expect("valid test fixture");
    let DeliveryCommand::Focus(focus) = receiver.try_control().expect("valid test fixture") else {
        panic!("focus command");
    };

    assert_eq!(focus.generation.covers_from_value(), Some(second));
}

fn focus(post: &str) -> DeliveryFocus {
    DeliveryFocus::compatibility(
        vec![FocusItem {
            post: PostId::new(post),
            meta: VideoMeta {
                urls: vec![format!("https://media.example/{post}.mp4")],
                delivery: DeliveryKind::Progressive,
                sha256: None,
                size_bytes: Some(16),
                duration_ms: Some(1_000),
            },
        }],
        0,
        0,
    )
}
