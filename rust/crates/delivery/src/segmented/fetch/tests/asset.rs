use super::super::axiom_test_support::{fetch, FetchInput};
use super::super::{FetchFailure, FetchSpec, FetchedObject};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_hls_manifest::hls_manifest::MAX_HLS_ASSET_BYTES;
use ghostr_net::media_request_executor::MediaRequestExecutor;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;
use url::Url;

pub(in crate::segmented::fetch) async fn asset(
    requests: &MediaRequestExecutor,
    url: &Url,
    priority: PreemptionAuthority,
) -> Result<FetchedObject, FetchFailure> {
    asset_with_timeouts(
        requests,
        url.as_str(),
        HlsTransferTimeouts::default(),
        priority,
    )
    .await
}

pub(in crate::segmented::fetch) async fn asset_with_timeouts(
    requests: &MediaRequestExecutor,
    url: &str,
    timeouts: HlsTransferTimeouts,
    priority: PreemptionAuthority,
) -> Result<FetchedObject, FetchFailure> {
    let network = crate::delivery_events::DeliveryNetworkStatusReader::new(
        crate::delivery_events::DeliveryNetworkStatus::unavailable(),
    );
    fetch(
        requests,
        FetchInput {
            spec: FetchSpec {
                url,
                limit: MAX_HLS_ASSET_BYTES,
                object_limit: MAX_HLS_ASSET_BYTES as u64,
                object: Default::default(),
                timeouts,
                priority,
                admission_fence: None,
            },
            traffic: None,
        },
        &network,
        None,
    )
    .await
}
