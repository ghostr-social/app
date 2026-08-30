mod support;

use core::time::Duration;
use rust_lib_ghostr::api::delivery_types::{FfiFocusItem, FfiMediaDelivery};
use rust_lib_ghostr::api::focus_control::{ffi_update_focus, FfiFocusTransition, FfiFocusUpdate};
use rust_lib_ghostr::engine::representation::RepresentationId;
use rust_lib_ghostr::engine::{DeliveryKind, VideoMeta};
use rust_lib_ghostr::video::video::{
    ffi_acquire_hls_playback, ffi_release_hls_playback, FfiHlsPreparedAssetAuthority,
};
use support::{engine, fixtures::temp_directory, hls_prepared_origin};

#[tokio::test]
async fn ffi_returns_the_exact_prepared_hls_authority() {
    let source = hls_prepared_origin::start().await;
    let directory = temp_directory("ghostr-hls-ffi-authority");
    engine::start_with_device_origin(&directory, 1_048_576, hls_prepared_origin::origin(&source))
        .await
        .expect("gateway endpoint");
    ffi_update_focus(focus(&source)).await.expect("HLS focus");
    let representation_id = representation(&source);
    let expected = FfiHlsPreparedAssetAuthority {
        delivery_id: "stream".to_owned(),
        representation_id: representation_id.clone(),
        asset_revision: 1,
    };

    let session = acquire_when_ready(expected, source).await;

    assert_eq!(session.delivery_id.as_deref(), Some("stream"));
    assert_eq!(session.representation_id, Some(representation_id));
    assert_eq!(session.asset_revision, Some(1));
    assert!(ffi_release_hls_playback(session.session_id).await);
    std::fs::remove_dir_all(directory).ok();
}

async fn acquire_when_ready(
    authority: FfiHlsPreparedAssetAuthority,
    source: String,
) -> rust_lib_ghostr::video::video::FfiHlsPlaybackSession {
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            match ffi_acquire_hls_playback(Some(authority.clone()), vec![source.clone()]).await {
                Ok(session) => return session,
                Err(_) => tokio::time::sleep(Duration::from_millis(20)).await,
            }
        }
    })
    .await
    .expect("prepared FFI authority deadline")
}

fn focus(source: &str) -> FfiFocusUpdate {
    FfiFocusUpdate {
        feed_id: "feed".to_owned(),
        items: vec![FfiFocusItem {
            post_id: "stream".to_owned(),
            urls: vec![source.to_owned()],
            delivery: FfiMediaDelivery::Hls,
            sha256: None,
            size_bytes: None,
            duration_ms: Some(4_000),
            blurhash: None,
        }],
        current_index: 0,
        watch_ms: 0,
        generation: 1,
        transition: FfiFocusTransition::UserNavigation,
        rescue: None,
    }
}

fn representation(source: &str) -> String {
    RepresentationId::for_meta(&VideoMeta {
        urls: vec![source.to_owned()],
        delivery: DeliveryKind::Hls,
        sha256: None,
        size_bytes: None,
        duration_ms: Some(4_000),
    })
    .fingerprint()
    .to_owned()
}
