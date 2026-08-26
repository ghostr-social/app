use super::{finished, run_work, Attempt, CpuClock, Run};
use crate::transform::{
    TransformBackend, TransformControl, TransformInput, TransformLimits, TransformOutput,
    TransformProfile,
};
use core::time::Duration;
use ghostr_engine::adaptive::TransformKind;
use std::sync::Arc;
use std::time::Instant;

struct SleepingBackend;

impl TransformBackend for SleepingBackend {
    fn profile(&self) -> TransformProfile {
        TransformProfile::new(
            TransformKind::Remux,
            TransformLimits::try_new(16, 16, 20, 5_000).expect("valid test fixture"),
        )
    }

    fn transform(
        &self,
        input: TransformInput<'_>,
        _control: &TransformControl,
    ) -> anyhow::Result<TransformOutput> {
        std::thread::sleep(Duration::from_millis(20));
        TransformOutput::try_new(input.bytes().to_vec())
    }
}

#[test]
fn sleeping_transform_records_zero_cpu() {
    let Attempt::Finished { output, cpu_ms } = measured_attempt() else {
        panic!("system CPU clock must produce a measured attempt");
    };
    assert_eq!(output.expect("sleeping transform succeeds"), b"data");
    assert_eq!(cpu_ms, 0, "sleep is not CPU consumption");
}

fn measured_attempt() -> Attempt {
    let backend: Arc<dyn TransformBackend> = Arc::new(SleepingBackend);
    let profile = backend.profile();
    let run = Run {
        backend,
        bytes: b"data".to_vec(),
        kind: TransformKind::Remux,
        profile,
        control: TransformControl::new(Instant::now() + Duration::from_secs(5)),
        resources: crate::manager::transforms::resource_test_fixture::control(),
    };
    finished(run_work(&run, CpuClock::system()), profile)
}
