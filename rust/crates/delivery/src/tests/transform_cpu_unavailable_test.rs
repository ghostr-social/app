use super::backend::{self, Attempt, Run};
use crate::transform::{
    TransformBackend, TransformControl, TransformInput, TransformLimits, TransformOutput,
    TransformProfile,
};
use ghostr_engine::adaptive::TransformKind;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

struct PassBackend {
    calls: Arc<AtomicUsize>,
}

impl TransformBackend for PassBackend {
    fn profile(&self) -> TransformProfile {
        let limits = TransformLimits::try_new(16, 16, 20, 100).unwrap();
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
        calls: calls.clone(),
    });
    let profile = backend.profile();
    let run = Run {
        backend,
        bytes: b"data".to_vec(),
        kind: TransformKind::Remux,
        profile,
        control: TransformControl::new(Instant::now() + Duration::from_millis(100)),
        resources: super::super::resource_test_fixture::control(),
    };

    let attempt = backend::execute_without_clock(run).await;

    assert!(matches!(
        attempt,
        Attempt::UnmeasuredFailure("warp_transform_cpu_measurement_unavailable")
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}
