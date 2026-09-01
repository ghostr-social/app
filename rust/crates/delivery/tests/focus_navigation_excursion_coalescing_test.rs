use ghostr_delivery::delivery_events::{
    command_channel, DeliveryCommand, DeliveryFocus, FocusAdmission, FocusGeneration, FocusItem,
};
use ghostr_engine::{DataUsageLevel, DeliveryKind, PostId, VideoMeta};

#[test]
fn pending_navigation_excursion_retains_its_reverse_turn() {
    let retained = retained(&[0, 3, 2, 1, 0]);

    assert_eq!(retained.last(), Some(&0), "latest focus must win");
    assert!(retained.len() <= 2, "focus ingress must remain bounded");
    assert!(
        retained.windows(2).any(|pair| pair[0] > pair[1]),
        "coalescing erased the p3-to-p0 reverse navigation: {retained:?}"
    );
}

#[test]
fn monotonic_or_same_current_navigation_coalesces_to_the_latest() {
    assert_eq!(retained(&[0, 1, 2, 3]), [3]);
    assert_eq!(retained(&[2, 2, 2]), [2]);
}

#[test]
fn partial_forward_correction_keeps_the_prior_reverse_edge() {
    assert_eq!(retained(&[0, 4, 2, 3]), [4, 3]);
}

#[test]
fn correction_reaching_the_prior_high_point_drops_the_stale_turn() {
    assert_eq!(retained(&[0, 4, 2, 4]), [4]);
    assert_eq!(retained(&[0, 4, 2, 5]), [5]);
}

#[test]
fn reverse_edge_stays_after_an_earlier_non_focus_control() {
    let (handle, mut receiver) = command_channel();
    let items = roster();
    handle.update_focus(focus(items.clone(), 0, 1));
    handle.set_data_usage(DataUsageLevel::Conservative);
    handle.update_focus(focus(items.clone(), 3, 2));
    handle.update_focus(focus(items, 2, 3));
    let commands = core::iter::from_fn(|| receiver.try_control()).collect::<Vec<_>>();

    assert!(matches!(commands[0], DeliveryCommand::Config(_)));
    let indices = commands
        .into_iter()
        .filter_map(current_index)
        .collect::<Vec<_>>();
    assert_eq!(indices, [3, 2]);
}

fn retained(indices: &[usize]) -> Vec<usize> {
    let (handle, mut receiver) = command_channel();
    let items = roster();
    for (offset, current) in indices.iter().copied().enumerate() {
        let generation = u64::try_from(offset + 1).expect("small fixture");
        assert_eq!(
            handle.update_focus(focus(items.clone(), current, generation)),
            FocusAdmission::Accepted
        );
    }
    core::iter::from_fn(|| receiver.try_control())
        .filter_map(current_index)
        .collect()
}

fn focus(items: Vec<FocusItem>, current_index: usize, generation: u64) -> DeliveryFocus {
    let mut focus = DeliveryFocus::compatibility(items, current_index, 0);
    focus.generation = FocusGeneration::try_new(generation).expect("positive generation");
    focus
}

fn current_index(command: DeliveryCommand) -> Option<usize> {
    match command {
        DeliveryCommand::Focus(focus) => Some(focus.current_index),
        _ => None,
    }
}

fn roster() -> Vec<FocusItem> {
    (0..7).map(item).collect()
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
