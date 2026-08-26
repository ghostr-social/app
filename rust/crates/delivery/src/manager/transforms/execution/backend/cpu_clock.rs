use core::time::Duration;

#[derive(Clone, Copy)]
pub(super) struct CpuClock {
    read: fn() -> Option<Duration>,
}

impl CpuClock {
    pub(super) const fn system() -> Self {
        Self { read: thread_time }
    }

    pub(super) fn read(self) -> Option<Duration> {
        (self.read)()
    }
}

#[cfg(unix)]
fn thread_time() -> Option<Duration> {
    use nix::sys::time::TimeValLike as _;

    let value = nix::time::clock_gettime(nix::time::ClockId::CLOCK_THREAD_CPUTIME_ID).ok()?;
    let nanos = u64::try_from(value.num_nanoseconds()).ok()?;
    Some(Duration::from_nanos(nanos))
}

#[cfg(not(unix))]
const fn thread_time() -> Option<Duration> {
    None
}

pub(super) fn elapsed(start: Option<Duration>, end: Option<Duration>) -> Option<Duration> {
    end?.checked_sub(start?)
}

#[cfg(test)]
#[path = "cpu_clock_axiom_test.rs"]
mod axiom_test_support;
