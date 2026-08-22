use ghostr_delivery::transform::{
    TransformBackend, TransformControl, TransformInput, TransformLimits, TransformOutput,
    TransformProfile, TransformTrigger,
};
use ghostr_engine::adaptive::TransformKind;
use std::hint::black_box;
use std::time::Duration;

const INPUT: &[u8] = b"ftyp|mdat:priced|moov:index";

pub struct PricedRemux;

impl TransformBackend for PricedRemux {
    fn profile(&self) -> TransformProfile {
        let limits = TransformLimits::try_new(1_024, 1_024, 500, 5_000).unwrap();
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
    let mut value = std::mem::MaybeUninit::<libc::timespec>::uninit();
    // SAFETY: The pointer is valid for one timespec and read only on success.
    let status = unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, value.as_mut_ptr()) };
    anyhow::ensure!(status == 0, "thread CPU clock unavailable");
    // SAFETY: A successful call initialized the entire timespec.
    let value = unsafe { value.assume_init() };
    let seconds = u64::try_from(value.tv_sec)?;
    let nanos = u32::try_from(value.tv_nsec)?;
    Ok(Duration::new(seconds, nanos))
}
