use crate::range_fixture;
use core::time::Duration;
use ghostr_delivery::chunk::downloader::{
    download_chunk_observed, ChunkExecution, ChunkSink, ChunkSpec, DownloadTraffic,
};
use ghostr_delivery::probe::media::{probe, ProbeSpec};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::host_stats::host_of;
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;

#[path = "fixture/environment.rs"]
mod environment;
use environment::Fixture;

pub(super) struct SurfaceTimeouts {
    pub progressive: anyhow::Error,
    pub head: anyhow::Error,
    pub failure_ratio: f64,
}

impl Fixture {
    async fn progressive(&mut self) -> anyhow::Error {
        let spec = ChunkSpec {
            requests: &self.requests,
            url: &self.url,
            request: range_fixture::range_request(ByteRange::new(0, 8)),
            attempt_profile: range_fixture::range_profile(8),
            priority: PreemptionAuthority::Transition,
            continuation: None,
            timeouts: self.timeouts,
        };
        let sink = ChunkSink {
            store: &self.store,
            key: "clip",
        };
        download_chunk_observed(
            &spec,
            ChunkExecution {
                sink: &sink,
                stats: &mut self.stats,
                cancel: &self.token,
                network: &self.network,
                traffic: &mut IgnoreTraffic,
                network_class: ghostr_engine::origin_model::NetworkClass::Unavailable,
            },
        )
        .await
        .result
        .expect_err("body admission must expire")
    }

    async fn head(&mut self) -> anyhow::Error {
        let spec = ProbeSpec {
            requests: &self.requests,
            url: &self.url,
            priority: PreemptionAuthority::Transition,
            timeouts: self.timeouts,
            network: None,
            profile: range_fixture::head_profile(),
        };
        probe(spec, &mut self.stats)
            .await
            .outcome
            .expect_err("HEAD admission must expire")
    }
}
pub(super) async fn exercise() -> SurfaceTimeouts {
    let mut fixture = Fixture::new().await;
    let progressive = fixture.progressive().await;
    let head = fixture.head().await;
    let host = host_of(&fixture.url).expect("host");
    let failure_ratio = fixture.stats.failure_ratio(&host);
    assert!(
        tokio::time::timeout(Duration::from_millis(50), fixture.listener.accept())
            .await
            .is_err(),
        "admission timeouts must occur before either surface reaches the origin"
    );
    std::fs::remove_dir_all(fixture.root).ok();
    SurfaceTimeouts {
        progressive,
        head,
        failure_ratio,
    }
}
pub(super) fn short_admission() -> TransferTimeouts {
    TransferTimeouts {
        admission: Duration::from_millis(20),
        headers: Duration::from_secs(1),
        idle: Duration::from_secs(1),
    }
}

struct IgnoreTraffic;

impl DownloadTraffic for IgnoreTraffic {
    fn opened(&mut self, _ttfb: Duration) {}
    fn wrote(&mut self, _bytes: u64) {}
}
