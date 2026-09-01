use ghostr_delivery::delivery_events::{
    command_channel, DeliveryCommand, DeliveryFocus, FocusGeneration, FocusItem, FocusTransition,
    TransportRescue, TransportRescueReason,
};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn third_semantic_transition_cannot_erase_a_pending_reverse_edge() {
    let (handle, mut receiver) = command_channel();
    for (generation, current) in [(1, 0), (2, 3), (3, 2)] {
        handle.update_focus(focus(generation, current, FocusTransition::UserNavigation));
    }
    handle.update_focus(focus(4, 2, FocusTransition::TransportRescue));
    let retained = core::iter::from_fn(|| receiver.try_control())
        .filter_map(as_focus)
        .collect::<Vec<_>>();

    assert!(retained.len() <= 3, "focus mailbox must remain bounded");
    assert!(
        retained.windows(2).any(|pair| pair[0].0 > pair[1].0),
        "third transition erased the reverse edge: {retained:?}"
    );
    assert_eq!(
        retained.last().map(|focus| focus.1),
        Some(FocusTransition::TransportRescue)
    );
}

fn as_focus(command: DeliveryCommand) -> Option<(usize, FocusTransition)> {
    let DeliveryCommand::Focus(focus) = command else {
        return None;
    };
    Some((focus.current_index, focus.transition))
}

fn focus(generation: u64, current_index: usize, transition: FocusTransition) -> DeliveryFocus {
    let rescue = (transition == FocusTransition::TransportRescue).then_some(TransportRescue {
        reason: TransportRescueReason::GraceExpired,
        rank_displacement: 1,
        wait_ms: 25,
    });
    DeliveryFocus {
        items: (0..5).map(item).collect(),
        previews: Vec::new(),
        current_index,
        watch_ms: 0,
        generation: FocusGeneration::try_new(generation).expect("positive generation"),
        transition,
        rescue,
    }
}

fn item(index: usize) -> FocusItem {
    let id = format!("p{index}");
    FocusItem {
        post: PostId::new(&id),
        meta: VideoMeta {
            urls: vec![format!("https://media.example/{id}.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(32),
            duration_ms: Some(4_000),
        },
    }
}
