use super::short_admission;
use crate::range_fixture;
use ghostr_delivery::chunk::cancel::{cancel_pair, CancelToken};
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::host_stats::HostStats;
use ghostr_net::media_request_executor::{
    AdmittedMediaRequest, MediaRequestExecutor, MediaRequestLimits,
};
use ghostr_net::transfer_timeouts::TransferTimeouts;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use tokio::net::TcpListener;

pub(super) struct Fixture {
    pub listener: TcpListener,
    pub url: String,
    pub requests: MediaRequestExecutor,
    pub _held: AdmittedMediaRequest,
    pub root: PathBuf,
    pub store: PartialRangeStore,
    pub token: CancelToken,
    pub network: NetworkThrottle,
    pub stats: HostStats,
    pub timeouts: TransferTimeouts,
}

impl Fixture {
    pub async fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
        let url = format!("http://{}/video.mp4", listener.local_addr().unwrap());
        let requests = MediaRequestExecutor::new(
            range_fixture::raw_media_client(),
            MediaRequestLimits::try_new(1, 1).unwrap(),
        );
        let held = requests
            .get(&url, PreemptionAuthority::Transition)
            .unwrap()
            .admit()
            .await
            .unwrap();
        let root = range_fixture::temp_root("surface-admission-timeout");
        let (_handle, token) = cancel_pair();
        Self {
            listener,
            url,
            requests,
            _held: held,
            store: range_fixture::store(root.clone()),
            root,
            token,
            network: range_fixture::network(),
            stats: HostStats::new(),
            timeouts: short_admission(),
        }
    }
}
