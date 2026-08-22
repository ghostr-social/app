use super::preparation_route_fixture::{PreparationRouteFixture, CURRENT_BYTES, NEXT_BYTES};

#[tokio::test]
async fn every_prepared_asset_url_serves_its_exact_cached_bytes() {
    let mut fixture = PreparationRouteFixture::start().await;
    let plan = fixture.next_plan().await;
    let current = plan.current.expect("current asset");
    let next = plan.next.expect("next asset");

    let current_response = fixture.get(&current.playback_url).await;
    let next_response = fixture.get(&next.playback_url).await;

    assert_eq!(current_response.0, reqwest::StatusCode::OK);
    assert_eq!(current_response.1, CURRENT_BYTES);
    assert_eq!(next_response.0, reqwest::StatusCode::OK);
    assert_eq!(next_response.1, NEXT_BYTES);
}
