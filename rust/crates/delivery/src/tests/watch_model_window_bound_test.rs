use crate::delivery_events::DeliveryFocus;
use crate::qoe::WatchLearner;
use crate::tests::watch_model_fixture::item;
use ghostr_engine::PostId;

#[test]
fn watch_metadata_tracks_only_the_current_and_bounded_forward_window() {
    let mut learner = WatchLearner::default();
    let items = (0..200).map(|index| item(&format!("p{index}"))).collect();
    learner.focus(&DeliveryFocus::compatibility(items, 50, 0), 0);
    let window = learner
        .window()
        .for_current(&PostId::new("p50"))
        .expect("current metadata");
    assert_eq!(
        retained_count(&learner),
        25,
        "only the bounded window is retained"
    );
    assert_eq!(window.len(), 25, "watch metadata has a fixed bound");
    assert_eq!(window[0].post, PostId::new("p50"), "current is first");
    assert_eq!(
        window[24].post,
        PostId::new("p74"),
        "the bounded future remains ordered"
    );
    learner.focus(&DeliveryFocus::compatibility(Vec::new(), 0, 0), 1);
    assert_eq!(
        retained_count(&learner),
        0,
        "clearing focus releases all window identities"
    );
}

fn retained_count(learner: &WatchLearner) -> usize {
    (0..200)
        .filter(|index| {
            learner
                .window()
                .for_current(&PostId::new(format!("p{index}")))
                .is_some()
        })
        .count()
}
