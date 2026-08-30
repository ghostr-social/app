use super::ProvisionalFocusHandoff;
use ghostr_engine::PostId;

#[test]
fn scope_admits_only_pre_cutoff_futures_for_the_same_current() {
    let current = PostId::new("current");
    let next = PostId::new("next");
    let third = PostId::new("third");
    let mut handoff = ProvisionalFocusHandoff::default();
    handoff.begin(
        Some(current.clone()),
        vec![next.clone(), third.clone()],
        1_000,
    );

    assert_eq!(handoff.rank(Some(&current), &next, 1_000), Some(0));
    assert_eq!(handoff.rank(Some(&current), &third, 1_000), Some(1));
    assert_eq!(handoff.rank(Some(&current), &next, 1_001), None);
    assert_eq!(
        handoff.rank(Some(&PostId::new("other")), &next, 1_000),
        None
    );

    assert_eq!(handoff.rank(Some(&current), &next, 1_000), Some(0));
    assert_eq!(handoff.rank(Some(&current), &third, 1_000), Some(1));
}
