use super::{Scenario, CHUNK_BYTES};
use crate::delivery_fixture::full_recovery_origin::{ObservedRequest, PROBE_BYTES, TRIAL_BYTES};
use crate::delivery_fixture::full_recovery_stats::{assert_admission, wait_for_admission};
use crate::delivery_fixture::items::{focus_now, sized_item};
use ghostr_engine::origin_model::Admission;

impl Scenario {
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
        assert_eq!(
            fallback.method.as_str(),
            "GET",
            "parallel fallback uses GET"
        );
        assert_eq!(
            fallback.path, "/parallel.mp4",
            "parallel fallback selects its source"
        );
        assert_eq!(
            fallback.range.as_deref(),
            Some("bytes=0-4095"),
            "parallel range stays bounded"
        );
        self.assert_method_specific_lease().await;
        probe.finish(PROBE_BYTES).await;
        fallback.finish(CHUNK_BYTES as usize).await;
    }

    pub(super) async fn start_trial(&mut self) -> ObservedRequest {
        let path = self.stats_path();
        let query = self.trial_query();
        wait_for_admission(&path, &query, Admission::RecoveryTrial).await;
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
        let trial = self.next_trial_request().await;
        assert_eq!(
            trial.encoding.as_deref(),
            Some("identity"),
            "trial uses identity encoding"
        );
        trial
    }

    pub(super) async fn assert_trial_open(&mut self, trial: &ObservedRequest) {
        trial.send(1).await;
        self.assert_no_competing_trial().await;
        self.assert_trial_lease();
        assert_admission(
            &self.stats_path(),
            &self.trial_query(),
            Admission::RecoveryTrial,
        )
        .await;
    }

    pub(super) async fn finish_trial(&self, trial: ObservedRequest) {
        let path = self.stats_path();
        let query = self.trial_query();
        trial.finish(TRIAL_BYTES - 1).await;
        wait_for_admission(&path, &query, Admission::Production).await;
        crate::delivery_fixture::wait::wait_for_file(&self.harness.root.join("trial.video")).await;
        self.harness.handle.clear().await.expect("clear manager");
        std::fs::remove_dir_all(&self.harness.root).ok();
    }
}
