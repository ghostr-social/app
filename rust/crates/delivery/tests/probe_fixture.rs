use ghostr_delivery::probe::media::{probe as probe_media, ProbeResult, ProbeSpec};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::origin_model::{
    MediaClass, OriginAttemptProfile, OriginRequestProfile, RequestMethod,
};
use ghostr_net::media_request_executor::MediaRequestExecutor;
use ghostr_net::transfer_timeouts::TransferTimeouts;

/// Runs one media probe through the shared integration-test configuration.
///
/// # Errors
///
/// Returns the admission, transport, or response-validation failure from the probe.
pub async fn probe(
    requests: &MediaRequestExecutor,
    url: &str,
    timeouts: TransferTimeouts,
    stats: &mut HostStats,
) -> anyhow::Result<ProbeResult> {
    probe_media(
        ProbeSpec {
            requests,
            url,
            priority: PreemptionAuthority::Transition,
            timeouts,
            network: None,
            profile: OriginAttemptProfile::new(OriginRequestProfile::new(
                RequestMethod::Head,
                0,
                MediaClass::Unknown,
            )),
        },
        stats,
    )
    .await
    .outcome
}
