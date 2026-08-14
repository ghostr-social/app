mod gateway_fixture;

use gateway_fixture::progressive_journey_origin::ProgressiveJourneyOrigin;

#[tokio::test]
async fn progressive_journey_uses_the_qoe_media_span() {
    let origin = ProgressiveJourneyOrigin::with_blocked_head().await;

    assert_eq!(origin.total_bytes(), 285_652);
    assert_eq!(
        origin.sha256(),
        "f4b18e44d7705cb706699a35e8179fe11b682ef09cba70fb320dba162e50f7e0",
    );
}
