mod gateway_fixture;

use gateway_fixture::progressive_journey_origin::ProgressiveJourneyOrigin;

#[tokio::test]
async fn progressive_journey_uses_the_qoe_media_span() {
    let origin = ProgressiveJourneyOrigin::with_blocked_head().await;

    assert_eq!(origin.total_bytes(), 285_652);
    assert_eq!(
        origin.sha256(),
        "74ddab015133a0fdb579a04fb71eb2a9b142629fce6eb55e9e87f8cf91d9592b",
    );
}
