use crate::manager::timeline::TimelineSchedule;
use crate::tests::timeline_cancellation_parser::CancellationHarness;

#[tokio::test]
async fn cancelled_parse_frees_its_slot_and_is_never_terminally_memoized() {
    let mut fixture = CancellationHarness::new().await;

    assert_eq!(
        fixture
            .coordinator
            .schedule(fixture.post.clone(), fixture.evidence.clone()),
        TimelineSchedule::Started
    );
    fixture
        .coordinator
        .dispatch(core::slice::from_ref(&fixture.post));
    assert_eq!(fixture.next_started().await, 0);
    fixture.coordinator.invalidate(&fixture.post);
    let cancelled = fixture.next_result().await;
    assert!(fixture
        .coordinator
        .validate(cancelled, Some(&fixture.evidence))
        .is_none());

    assert_eq!(
        fixture
            .coordinator
            .schedule(fixture.post.clone(), fixture.evidence.clone()),
        TimelineSchedule::Started
    );
    fixture
        .coordinator
        .dispatch(core::slice::from_ref(&fixture.post));
    assert_eq!(fixture.next_started().await, 1);
    assert_eq!(fixture.next_result().await.post(), &fixture.post);
    tokio::fs::remove_dir_all(fixture.root).await.expect("valid test fixture");
}
