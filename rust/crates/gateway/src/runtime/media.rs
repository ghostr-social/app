#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
use crate::debug::media::DebugMediaHttpClient;
#[cfg(all(feature = "device-integration", debug_assertions))]
use crate::device_integration::DeviceIntegrationMediaHttpClient;
use crate::runtime::GatewayConfiguration;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::{MediaHttpClient, MediaHttpRequests};
use std::sync::Arc;

#[cfg(test)]
mod allowance_restart_test;

pub(super) fn client(
    configuration: &GatewayConfiguration,
) -> anyhow::Result<Arc<dyn MediaHttpRequests>> {
    #[cfg(all(feature = "device-integration", debug_assertions))]
    if let Some(origin) = configuration.device_integration_origin.as_deref() {
        return Ok(Arc::new(DeviceIntegrationMediaHttpClient::new(origin)?));
    }
    anyhow::ensure!(
        configuration.device_integration_origin.is_none(),
        "device integration media is unavailable"
    );
    Ok(Arc::new(MediaHttpClient::public()?))
}

#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
pub(super) fn debug_client() -> anyhow::Result<Arc<dyn MediaHttpRequests>> {
    Ok(Arc::new(DebugMediaHttpClient::new()?))
}

pub(super) fn executor(
    client: Arc<dyn MediaHttpRequests>,
    maximum: usize,
    directory: &std::path::Path,
    data_limit: ghostr_net::internet_allowance::InternetDataLimit,
) -> anyhow::Result<MediaRequestExecutor> {
    let limits = MediaRequestLimits::try_new(maximum, maximum)?;
    let allowance = ghostr_net::internet_allowance::InternetAllowance::open(
        &directory.join("internet-allowance"),
        data_limit,
    )?;
    Ok(MediaRequestExecutor::with_allowance(
        client, limits, allowance,
    ))
}
