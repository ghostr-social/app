use super::full_recovery_origin::{RecoveryOrigin, PROBE_BYTES, TRIAL_BYTES};
use super::full_recovery_stats::{query, seed, unix_time_ms};
use super::options::DeliveryOptions;
use super::{start_harness_at, temp_directory, DeliveryHarness};
use core::num::NonZeroUsize;
use ghostr_engine::origin_model::OriginQuery;
use std::path::PathBuf;

mod lease;
mod recovery;

pub(super) const CHUNK_BYTES: u64 = 4_096;

pub async fn run() {
    let mut scenario = Scenario::new().await;
    let probe = scenario.start_probe().await;
    scenario.complete_probe_with_parallel_work(probe).await;
    let trial = scenario.start_trial().await;
    scenario.assert_trial_open(&trial).await;
    scenario.finish_trial(trial).await;
}

pub(super) struct Scenario {
    pub(super) origin: RecoveryOrigin,
    pub(super) harness: DeliveryHarness,
    pub(super) probe_url: String,
    pub(super) parallel_url: String,
    pub(super) trial_url: String,
}

impl Scenario {
    async fn new() -> Self {
        let origin = RecoveryOrigin::serve().await;
        let probe_url = origin.url("/probe.mp4");
        let parallel_url = origin.url("/parallel.mp4");
        let trial_url = origin.url("/trial.mp4");
        let root = temp_directory("full-get-recovery");
        seed_urls(&root, &probe_url, &parallel_url, &trial_url);
        let harness = start_harness_at(root, options());
        Self {
            origin,
            harness,
            probe_url,
            parallel_url,
            trial_url,
        }
    }

    pub(super) fn stats_path(&self) -> PathBuf {
        self.harness.root.join("host_stats.json")
    }

    pub(super) fn trial_query(&self) -> OriginQuery {
        query(&self.trial_url, TRIAL_BYTES as u64, unix_time_ms())
    }
}

fn options() -> DeliveryOptions {
    let mut options = DeliveryOptions::default();
    options.params.balanced_concurrency = 2;
    options.params.chunk_bytes = CHUNK_BYTES;
    options.tuning.max_requests_per_authority = NonZeroUsize::new(2);
    options
}

fn seed_urls(root: &std::path::Path, probe: &str, parallel: &str, trial: &str) {
    seed(
        root,
        &[
            (probe, TRIAL_BYTES as u64),
            (parallel, PROBE_BYTES as u64),
            (trial, TRIAL_BYTES as u64),
        ],
    );
}
