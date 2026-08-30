mod gateway_fixture;

use core::time::Duration;
use gateway_fixture::delivery::start_delivery;
use gateway_fixture::hls_prepared_origin::PreparedHlsOrigin;
use gateway_fixture::hls_prepared_session::wait_authority;
use gateway_fixture::progressive_hls::hls_focus;
use ghostr_gateway::hls::playback::HlsPlaybackRequest;
use ghostr_gateway::hls::sessions::{HlsSessionLimits, HlsSessions};

#[tokio::test]
async fn prepared_authority_and_sources_share_one_bounded_admission() {
    let (_, source) = PreparedHlsOrigin::start("bounded").await;
    let delivery = start_delivery("hls-prepared-capacity");
    delivery.handle.update_focus(hls_focus(&source));
    let authority = wait_authority(&delivery.segmented, None).await;
    let limits = HlsSessionLimits::new(1, Duration::from_secs(60), 8).expect("limits");
    let sessions = HlsSessions::new(limits);
    let request = || {
        HlsPlaybackRequest::new(authority.clone(), vec![source.clone()])
            .expect("valid prepared playback request")
    };
    let unrelated = HlsPlaybackRequest::new(
        authority.clone(),
        vec!["https://unrelated.example/index.m3u8".to_owned()],
    )
    .expect("structurally valid request");
    assert!(sessions
        .acquire_prepared(&delivery.segmented, unrelated)
        .await
        .is_err());
    let first = sessions
        .acquire_prepared(&delivery.segmented, request())
        .await
        .expect("first session");

    assert!(sessions
        .acquire_prepared(&delivery.segmented, request())
        .await
        .is_err());
    assert_eq!(sessions.authority(&first).await, Some(authority.clone()));
    assert!(sessions.release(&first).await);
    assert!(sessions
        .acquire_prepared(&delivery.segmented, request())
        .await
        .is_ok());
}
