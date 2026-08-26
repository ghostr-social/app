use core::hint::black_box;
use core::time::Duration;
use ghostr_delivery::transform::{
    TransformBackend, TransformControl, TransformInput, TransformLimits, TransformOutput,
    TransformProfile, TransformTrigger,
};
use ghostr_engine::adaptive::TransformKind;

const INPUT: &[u8] = b"ftyp|mdat:priced|moov:index";

pub struct PricedRemux;

impl TransformBackend for PricedRemux {
    fn profile(&self) -> TransformProfile {
        let limits =
            TransformLimits::try_new(1_024, 1_024, 500, 5_000).expect("valid test fixture");
        TransformProfile::new(TransformKind::Remux, limits)
            .with_trigger(TransformTrigger::InvalidVideoTrack)
    }

    fn transform(
        &self,
        input: TransformInput<'_>,
        control: &TransformControl,
    ) -> anyhow::Result<TransformOutput> {
        control.checkpoint()?;
        anyhow::ensure!(input.bytes() == INPUT, "wrong transform input");
        consume_thread_cpu(Duration::from_millis(460), control)?;
        control.checkpoint()?;
        TransformOutput::try_new(input.bytes().to_vec())
    }
}

fn consume_thread_cpu(target: Duration, control: &TransformControl) -> anyhow::Result<()> {
    let started = thread_cpu_time()?;
    let mut value = 1_u64;
    while thread_cpu_time()?.saturating_sub(started) < target {
        value = black_box(value.wrapping_mul(31).wrapping_add(7));
        control.checkpoint()?;
    }
    black_box(value);
    Ok(())
}

fn thread_cpu_time() -> anyhow::Result<Duration> {
    use nix::sys::time::TimeValLike as _;

    let value = nix::time::clock_gettime(nix::time::ClockId::CLOCK_THREAD_CPUTIME_ID)?;
    let nanos = u64::try_from(value.num_nanoseconds())?;
    Ok(Duration::from_nanos(nanos))
}
