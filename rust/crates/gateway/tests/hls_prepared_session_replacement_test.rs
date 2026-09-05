mod gateway_fixture;

use gateway_fixture::delivery::start_delivery;
use gateway_fixture::hls_prepared_origin::PreparedHlsOrigin;
use gateway_fixture::hls_prepared_session::{bootstrap, wait_authority, BootstrapBodies};
use gateway_fixture::media_client;
use gateway_fixture::progressive_hls::{hls_focus, router_with_segmented_hls};
use ghostr_delivery::segmented::HlsPreparedAssetAuthority;
use ghostr_gateway::hls::playback::HlsPlaybackRequest;
use ghostr_gateway::hls::sessions::HlsSessions;

#[tokio::test]
async fn replacement_sessions_pin_disjoint_bootstrap_cohorts() {
    let (old_origin, old_source) = PreparedHlsOrigin::start("old").await;
    let (new_origin, new_source) = PreparedHlsOrigin::start("new").await;
    let delivery = start_delivery("hls-prepared-replacement");
    delivery.handle.update_focus(hls_focus(&old_source));
    let old = wait_authority(&delivery.segmented, None).await;
    let sessions = HlsSessions::production();
    let old_session = acquire(&sessions, &delivery.segmented, &old, &old_source).await;

    delivery.handle.update_focus(hls_focus(&new_source));
    let fresh = wait_authority(&delivery.segmented, Some(&old)).await;
    assert!(sessions
        .acquire_prepared(&delivery.segmented, request(&old, &old_source))
        .await
        .is_err());
    let fresh_session = acquire(&sessions, &delivery.segmented, &fresh, &new_source).await;

    let old_hits = old_origin.hits();
    let new_hits = new_origin.hits();
    let router =
        router_with_segmented_hls(sessions.clone(), media_client(), delivery.segmented.clone());
    assert_eq!(bootstrap(&router, &old_session).await, bodies("old"));
    assert_eq!(bootstrap(&router, &fresh_session).await, bodies("new"));
    assert_eq!(old_origin.hits(), old_hits, "old route refetched origin");
    assert_eq!(new_origin.hits(), new_hits, "new route refetched origin");
}

async fn acquire(
    sessions: &HlsSessions,
    cache: &ghostr_delivery::segmented::SegmentedCache,
    authority: &HlsPreparedAssetAuthority,
    source: &str,
) -> ghostr_gateway::hls::sessions::HlsSessionId {
    sessions
        .acquire_prepared(cache, request(authority, source))
        .await
        .expect("current prepared session")
}

fn request(authority: &HlsPreparedAssetAuthority, source: &str) -> HlsPlaybackRequest {
    HlsPlaybackRequest::new(authority.clone(), vec![source.to_owned()])
        .expect("valid prepared playback request")
}

fn bodies(version: &str) -> BootstrapBodies {
    BootstrapBodies {
        init: format!("{version}-init"),
        segment: format!("{version}-segment"),
    }
}
