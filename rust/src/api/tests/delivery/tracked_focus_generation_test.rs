use crate::api::runtime::tracked_items::TrackedItems;
use crate::api::tests::support::sized_meta;
use ghostr_delivery::delivery_events::FocusGeneration;

#[test]
fn older_focus_cannot_replace_newer_watched_items() {
    let tracked = TrackedItems::new();

    assert!(tracked.replace_focus(generation(2), vec![entry("new")]));
    assert!(!tracked.replace_focus(generation(1), vec![entry("old")]));

    assert_eq!(tracked.snapshot()[0].0, "new");
}

#[test]
fn unversioned_focus_cannot_replace_the_watched_items() {
    let tracked = TrackedItems::new();

    assert!(!tracked.replace_focus(FocusGeneration::compatibility(), vec![entry("old")]));
    assert!(tracked.snapshot().is_empty());
}

fn generation(value: u64) -> FocusGeneration {
    FocusGeneration::try_new(value).expect("positive generation")
}

fn entry(id: &str) -> (String, crate::engine::VideoMeta) {
    (id.to_owned(), sized_meta(16, 2_000))
}
