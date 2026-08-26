use super::*;
use crate::transform::{
    TransformBackend, TransformControl, TransformInput, TransformLimits, TransformOutput,
};
use core::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

struct PassBackend {
    calls: Arc<AtomicUsize>,
}

impl TransformBackend for PassBackend {
    fn profile(&self) -> TransformProfile {
        let limits = TransformLimits::try_new(16, 16, 20, 100).expect("valid test fixture");
        TransformProfile::new(TransformKind::Remux, limits)
    }

    fn transform(
        &self,
        input: TransformInput<'_>,
        _control: &TransformControl,
    ) -> anyhow::Result<TransformOutput> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        TransformOutput::try_new(input.bytes().to_vec())
    }
}

#[tokio::test]
async fn unavailable_cpu_clock_fails_closed_before_output_publication() {
    let calls = Arc::new(AtomicUsize::new(0));
    let backend = Arc::new(PassBackend {
        calls: Arc::clone(&calls),
    });
    let run = Run {
        profile: backend.profile(),
        backend,
        bytes: b"data".to_vec(),
        kind: TransformKind::Remux,
        control: TransformControl::new(Instant::now() + Duration::from_millis(100)),
        resources: crate::manager::transforms::resource_test_fixture::control(),
    };

    let attempt = execute_with_clock(run, CpuClock::unavailable()).await;

    assert!(matches!(
        attempt,
        Attempt::UnmeasuredFailure("warp_transform_cpu_measurement_unavailable")
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}
