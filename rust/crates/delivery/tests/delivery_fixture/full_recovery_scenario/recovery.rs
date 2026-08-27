use super::{unix_time_ms, Scenario, CHUNK_BYTES};
use crate::delivery_fixture::full_recovery_origin::{ObservedRequest, PROBE_BYTES, TRIAL_BYTES};
use crate::delivery_fixture::full_recovery_stats::assert_admission;
use crate::delivery_fixture::items::{focus_now, sized_item};
use crate::delivery_fixture::stats::wait_for;
use ghostr_engine::origin_model::Admission;

impl Scenario {
    pub(super) async fn start_probe(&mut self) -> ObservedRequest {
        self.harness.handle.update_focus(focus_now(
            vec![sized_item(
                "probe",
                &self.probe_url,
                TRIAL_BYTES as u64,
                10_000,
            )],
            0,
            0,
        ));
        let probe = self.origin.next_within("capped probe").await;
        assert_eq!(probe.method.as_str(), "GET");
        assert_eq!(probe.path, "/probe.mp4");
        assert_eq!(probe.range.as_deref(), Some("bytes=0-65535"));
        assert_eq!(probe.encoding.as_deref(), Some("identity"));
        probe
    }

    pub(super) async fn complete_probe_with_parallel_work(&mut self, probe: ObservedRequest) {
        self.harness.handle.update_focus(focus_now(
            vec![
                sized_item("probe", &self.probe_url, TRIAL_BYTES as u64, 10_000),
                sized_item("parallel", &self.parallel_url, PROBE_BYTES as u64, 1_000),
            ],
            0,
            0,
        ));
        let fallback = self.origin.next_within("parallel range").await;
        assert_eq!(fallback.method.as_str(), "GET");
        assert_eq!(fallback.path, "/parallel.mp4");
        assert_eq!(fallback.range.as_deref(), Some("bytes=0-4095"));
        self.assert_method_specific_lease().await;
        probe.finish(PROBE_BYTES).await;
        fallback.finish(CHUNK_BYTES as usize).await;
    }

    pub(super) async fn start_trial(&mut self) -> ObservedRequest {
        let path = self.stats_path();
        let query = self.trial_query();
        wait_for(&path, |stats| {
            stats
                .origin_model()
                .circuit_admission(&query, unix_time_ms())
                == Admission::RecoveryTrial
        })
        .await;
        self.harness.handle.update_focus(focus_now(
            vec![sized_item(
                "trial",
                &self.trial_url,
                TRIAL_BYTES as u64,
                1_000,
            )],
            0,
            0,
        ));
        let trial = self.origin.next_within("Full recovery trial").await;
        assert_eq!(trial.method.as_str(), "GET");
        assert_eq!(trial.path, "/trial.mp4");
        assert_eq!(trial.range, None);
        assert_eq!(trial.encoding.as_deref(), Some("identity"));
        trial
    }

    pub(super) async fn assert_trial_open(&mut self, trial: &ObservedRequest) {
        trial.send(1).await;
        self.origin.assert_quiet().await;
        assert_admission(
            &self.stats_path(),
            &self.trial_query(),
            Admission::RecoveryTrial,
        )
        .await;
    }

    pub(super) async fn finish_trial(&mut self, trial: ObservedRequest) {
        let path = self.stats_path();
        let query = self.trial_query();
        trial.finish(TRIAL_BYTES - 1).await;
        wait_for(&path, |stats| {
            stats
                .origin_model()
                .circuit_admission(&query, unix_time_ms())
                == Admission::Production
        })
        .await;
        crate::delivery_fixture::wait::wait_for_file(&self.harness.root.join("trial.video")).await;
        self.harness.handle.clear().await.expect("clear manager");
        std::fs::remove_dir_all(&self.harness.root).ok();
    }
}
