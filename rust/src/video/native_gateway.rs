use crate::api::runtime::registry;
use crate::video::ffi_models::ffi_hls_playback_session;
use anyhow::Context as _;
use flutter_rust_bridge::frb;
use ghostr_gateway::hls::playback::HlsPlaybackRequest;

pub use crate::video::ffi_models::{FfiHlsPlaybackSession, FfiHlsPreparedAssetAuthority};

#[frb]
pub async fn ffi_acquire_hls_playback(
    authority: Option<FfiHlsPreparedAssetAuthority>,
    source_urls: Vec<String>,
) -> anyhow::Result<FfiHlsPlaybackSession> {
    acquire_hls_playback(authority, source_urls).await
}

async fn acquire_hls_playback(
    authority: Option<FfiHlsPreparedAssetAuthority>,
    source_urls: Vec<String>,
) -> anyhow::Result<FfiHlsPlaybackSession> {
    let engine = registry::engine()?;
    let session = match authority {
        Some(authority) => {
            let native = engine
                .gateway
                .segmented()
                .resolve_prepared_authority(
                    &authority.delivery_id,
                    &authority.representation_id,
                    authority.asset_revision,
                )
                .context("prepared HLS authority is unavailable")?;
            let request = HlsPlaybackRequest::new(native, source_urls)?;
            engine.gateway.acquire_hls(request).await?
        }
        None => engine.gateway.acquire_unprepared_hls(source_urls).await?,
    };
    Ok(ffi_hls_playback_session(session))
}

#[frb]
pub async fn ffi_release_hls_playback(session_id: String) -> bool {
    release_hls_playback(&session_id).await
}

async fn release_hls_playback(session_id: &str) -> bool {
    match registry::engine_if_running() {
        Some(engine) => engine.gateway.release_hls(session_id).await,
        None => false,
    }
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    #[cfg(target_os = "android")]
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Trace)
            .with_filter(
                android_logger::FilterBuilder::new()
                    .parse("debug,mp4parse=off,nostr_relay_pool=off,hyper_util=off,reqwest=off")
                    .build(),
            ),
    );
}
