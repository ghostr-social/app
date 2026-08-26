use core::{hint::black_box, time::Duration};
use ghostr_delivery::transform::{
    TransformBackend, TransformControl, TransformInput, TransformLimits, TransformOutput,
    TransformProfile, TransformTrigger,
};
use ghostr_engine::adaptive::TransformKind;
use std::time::Instant;

pub struct RejectedRemux;

impl TransformBackend for RejectedRemux {
    fn profile(&self) -> TransformProfile {
        let limits =
            TransformLimits::try_new(1_024, 1_024, 500, 1_000).expect("valid test fixture");
        TransformProfile::new(TransformKind::Remux, limits)
            .with_trigger(TransformTrigger::InvalidVideoTrack)
    }

    fn transform(
        &self,
        _input: TransformInput<'_>,
        _control: &TransformControl,
    ) -> anyhow::Result<TransformOutput> {
        let started = Instant::now();
        let mut value = 1_u64;
        while started.elapsed() < Duration::from_millis(10) {
            value = black_box(value.wrapping_mul(31).wrapping_add(7));
        }
        black_box(value);
        anyhow::bail!("fixture rejection")
    }
}
