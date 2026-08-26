use super::range_fixture;
use ghostr_delivery::chunk::cancel::{cancel_pair, CancelToken};
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_net::media_request_executor::{AdmittedMediaRequest, MediaRequestExecutor};
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;

mod run;
mod support;
use support::{admit, executor, expect_queued, spec, ObservedTraffic, BODY_BYTES};

pub(super) struct Observation {
    pub concurrency: usize,
    pub throughput: f64,
}

struct ObservationFixture {
    url: String,
    requests: MediaRequestExecutor,
    _held_same: AdmittedMediaRequest,
    held_other: Option<AdmittedMediaRequest>,
    root: PathBuf,
    store: PartialRangeStore,
    token: CancelToken,
    network: NetworkThrottle,
}

impl ObservationFixture {
    async fn new() -> Self {
        let url = range_fixture::ranged::serve_ranged(vec![7; BODY_BYTES as usize]).await;
        let other = range_fixture::ranged::serve_ranged(vec![8]).await;
        let requests = executor();
        let held_same = admit(&requests, &url).await;
        let held_other = admit(&requests, &other).await;
        let root = range_fixture::temp_root("admission-observation");
        let store = range_fixture::store(root.clone());
        let (_handle, token) = cancel_pair();
        Self {
            url,
            requests,
            _held_same: held_same,
            held_other: Some(held_other),
            root,
            store,
            token,
            network: range_fixture::network(),
        }
    }

    async fn observe(mut self) -> Observation {
        let mut stats = HostStats::new();
        let mut traffic = ObservedTraffic::default();
        let host = host_of(&self.url).expect("valid test fixture");
        run::download(&mut self, &mut stats, &mut traffic).await;
        let throughput = stats.expected_throughput(&host);
        let concurrency = traffic.concurrency;
        std::fs::remove_dir_all(self.root).ok();
        Observation {
            concurrency,
            throughput,
        }
    }
}

pub(super) async fn observe() -> Observation {
    ObservationFixture::new().await.observe().await
}
