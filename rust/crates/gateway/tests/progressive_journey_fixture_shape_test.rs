mod gateway_fixture;

use gateway_fixture::progressive_journey_origin::ProgressiveJourneyOrigin;

#[tokio::test]
async fn progressive_journey_uses_the_qoe_media_span() {
    let origin = ProgressiveJourneyOrigin::with_blocked_head().await;

    assert_eq!(origin.total_bytes(), 293_999);
    assert_eq!(
        origin.sha256(),
        "d5c6f10986d2a23730172e1b74e8ee7e9dd976aefe673570e719029ec3bd4ddc",
    );
}
