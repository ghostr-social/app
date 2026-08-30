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
async fn invalidated_authority_cannot_reopen_but_its_pinned_session_stays_coherent() {
    let (origin, source) = PreparedHlsOrigin::start("old").await;
    let delivery = start_delivery("hls-prepared-invalidation");
    delivery.handle.update_focus(hls_focus(&source));
    let old = wait_authority(&delivery.segmented, None).await;
    let sessions = HlsSessions::production();
    let old_session = sessions
        .acquire_prepared(&delivery.segmented, request(&old, &source))
        .await
        .expect("current prepared session");
    assert_eq!(sessions.authority(&old_session).await.as_ref(), Some(&old));

    let generation = delivery
        .segmented
        .object(&source)
        .expect("prepared root")
        .generation();
    origin.set_version("fresh");
    assert!(delivery
        .segmented
        .invalidate_generation(&source, generation));
    assert!(sessions
        .acquire_prepared(&delivery.segmented, request(&old, &source))
        .await
        .is_err());
    let fresh = wait_authority(&delivery.segmented, Some(&old)).await;
    let fresh_session = sessions
        .acquire_prepared(&delivery.segmented, request(&fresh, &source))
        .await
        .expect("republished prepared session");
    assert_ne!(old.asset_revision(), fresh.asset_revision());

    let hits = origin.hits();
    let router =
        router_with_segmented_hls(sessions.clone(), media_client(), delivery.segmented.clone());
    let old_expected = BootstrapBodies {
        init: "old-init".to_owned(),
        segment: "old-segment".to_owned(),
    };
    let fresh_expected = BootstrapBodies {
        init: "fresh-init".to_owned(),
        segment: "fresh-segment".to_owned(),
    };
    assert_eq!(bootstrap(&router, &old_session).await, old_expected);
    assert_eq!(bootstrap(&router, &fresh_session).await, fresh_expected);
    assert_eq!(origin.hits(), hits, "session routes must not refetch");
}

fn request(authority: &HlsPreparedAssetAuthority, source: &str) -> HlsPlaybackRequest {
    HlsPlaybackRequest::new(authority.clone(), vec![source.to_owned()])
        .expect("valid prepared playback request")
}
