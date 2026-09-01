use super::promotion_fixture::PromotionFixture;
use crate::manager::inflight::CompletionStatus;

#[tokio::test]
async fn terminal_completion_returns_the_committed_reservation_once() {
    let mut fixture = PromotionFixture::new(100).await;
    let first = fixture.active.finish_with_resources(&fixture.attempt);

    assert_eq!(first.status(), CompletionStatus::Current);
    let reservation = first.network_reservation().expect("valid test fixture");
    assert_eq!(reservation.committed_bytes(), 4);
    assert_eq!(reservation.actual_bytes(0), 0);

    let late = fixture.active.finish_with_resources(&fixture.attempt);
    assert_eq!(late.status(), CompletionStatus::Superseded);
    assert_eq!(late.network_reservation(), None);
    fixture.cleanup().await;
}

#[tokio::test]
async fn committed_promotion_delta_joins_the_terminal_reservation() {
    let mut fixture = PromotionFixture::new(100).await;
    fixture.observe_headers(50);
    let preflight = fixture
        .active
        .preflight_promotion(&fixture.target, 50)
        .expect("valid test fixture");
    assert!(fixture.active.activate_promotion(&preflight, 50));
    assert!(fixture.active.commit_promotion_network(&preflight));

    let finished = fixture.active.finish_with_resources(&fixture.attempt);
    let reservation = finished.network_reservation().expect("valid test fixture");
    assert_eq!(reservation.committed_bytes(), 16);
    assert_eq!(reservation.actual_bytes(17), 17);
    fixture.cleanup().await;
}

#[tokio::test]
async fn clear_keeps_the_reservation_until_cancelled_io_reports_done() {
    let mut fixture = PromotionFixture::new(100).await;
    fixture.active.clear();

    let finished = fixture.active.finish_with_resources(&fixture.attempt);
    assert_eq!(finished.status(), CompletionStatus::Cancelled);
    assert_eq!(
        finished
            .network_reservation()
            .expect("valid test fixture")
            .committed_bytes(),
        4
    );
    fixture.cleanup().await;
}
