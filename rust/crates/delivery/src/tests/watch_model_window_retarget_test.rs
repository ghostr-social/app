use crate::delivery_events::FocusTransition;
use crate::qoe::WatchLearner;
use crate::tests::watch_model_fixture::focus;
use ghostr_engine::PostId;

#[test]
fn a_retained_current_identity_selects_its_own_ordered_metadata_suffix() {
    let mut learner = WatchLearner::default();
    learner.focus(&focus(0, 0, FocusTransition::UserNavigation), 0);
    let suffix = learner
        .window()
        .for_current(&PostId::new("b"))
        .expect("retained metadata");
    assert_eq!(suffix.len(), 2, "only this video and its successors remain");
    assert_eq!(
        suffix[0].post,
        PostId::new("b"),
        "the selected current is first"
    );
    assert_eq!(
        suffix[1].post,
        PostId::new("c"),
        "successor order is preserved"
    );
    assert!(
        learner
            .window()
            .for_current(&PostId::new("missing"))
            .is_none(),
        "unretained identities cannot borrow another current's forecast"
    );
    assert_eq!(
        learner
            .window()
            .for_current(&PostId::new("a"))
            .expect("original current")[0]
            .post,
        PostId::new("a"),
        "reading a suffix does not mutate the focus window"
    );
}
