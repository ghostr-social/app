use ghostr_delivery::delivery_events::{
    command_channel, DeliveryCommand, DeliveryFocus, FocusGeneration, FocusItem, FocusTransition,
    TransportRescue, TransportRescueReason,
};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn a_burst_swipe_cannot_relabel_a_transport_failure_as_user_navigation() {
    let (handle, mut receiver) = command_channel();
    let rescue = transport_rescue();
    handle.update_focus(focus(1, 1, FocusTransition::TransportRescue, Some(rescue)));
    handle.update_focus(focus(2, 2, FocusTransition::UserNavigation, None));
    handle.update_focus(focus(2, 3, FocusTransition::RosterChange, None));
    let retained = focuses(&mut receiver);

    assert_eq!(retained.len(), 2, "focus burst stays bounded");
    assert_eq!(
        retained[1].items[retained[1].current_index].post,
        PostId::new("c")
    );
    assert_eq!(retained[0].transition, FocusTransition::TransportRescue);
    assert_eq!(retained[0].rescue, Some(rescue));
    assert_eq!(retained[1].transition, FocusTransition::UserNavigation);
    assert_eq!(retained[1].rescue, None);
}

#[test]
fn a_later_rescue_cannot_relabel_an_undrained_user_navigation() {
    let (handle, mut receiver) = command_channel();
    handle.update_focus(focus(1, 1, FocusTransition::UserNavigation, None));
    handle.update_focus(focus(
        2,
        2,
        FocusTransition::TransportRescue,
        Some(transport_rescue()),
    ));
    let retained = focuses(&mut receiver);

    assert_eq!(retained.len(), 2, "focus transition order is bounded");
    assert_eq!(retained[0].transition, FocusTransition::UserNavigation);
    assert_eq!(retained[0].rescue, None);
    assert_eq!(retained[1].transition, FocusTransition::TransportRescue);
    assert_eq!(retained[1].rescue, Some(transport_rescue()));
}

fn focuses(receiver: &mut ghostr_delivery::delivery_events::CommandReceiver) -> Vec<DeliveryFocus> {
    core::iter::from_fn(|| receiver.try_control())
        .map(|command| match command {
            DeliveryCommand::Focus(focus) => focus,
            _ => panic!("focus command"),
        })
        .collect()
}

fn transport_rescue() -> TransportRescue {
    TransportRescue {
        reason: TransportRescueReason::DeliveryFailed,
        rank_displacement: 1,
        wait_ms: 25,
    }
}

fn focus(
    index: usize,
    generation: u64,
    transition: FocusTransition,
    rescue: Option<TransportRescue>,
) -> DeliveryFocus {
    DeliveryFocus {
        items: ["a", "b", "c"].into_iter().map(item).collect(),
        previews: Vec::new(),
        current_index: index,
        watch_ms: 900,
        generation: FocusGeneration::try_new(generation).expect("positive generation"),
        transition,
        rescue,
    }
}

fn item(id: &str) -> FocusItem {
    FocusItem {
        post: PostId::new(id),
        meta: VideoMeta {
            urls: vec![format!("https://media.example/{id}.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(16),
            duration_ms: Some(1_000),
        },
    }
}
