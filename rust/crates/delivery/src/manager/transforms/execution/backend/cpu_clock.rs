use std::time::Duration;

#[derive(Clone, Copy)]
pub(super) struct CpuClock {
    read: fn() -> Option<Duration>,
}

impl CpuClock {
    pub(super) const fn system() -> Self {
        Self { read: thread_time }
    }

    #[cfg(test)]
    pub(super) const fn unavailable() -> Self {
        Self { read: || None }
    }

    pub(super) fn read(self) -> Option<Duration> {
        (self.read)()
    }
}

#[cfg(unix)]
fn thread_time() -> Option<Duration> {
    let mut value = std::mem::MaybeUninit::<libc::timespec>::uninit();
    // SAFETY: `value` points to writable storage for one `timespec`, and it is
    // read only after `clock_gettime` reports success.
    let status = unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, value.as_mut_ptr()) };
    if status != 0 {
        return None;
    }
    // SAFETY: A successful `clock_gettime` initialized the complete value.
    let value = unsafe { value.assume_init() };
    timespec_duration(value)
}

#[cfg(not(unix))]
const fn thread_time() -> Option<Duration> {
    None
}

pub(super) fn elapsed(start: Option<Duration>, end: Option<Duration>) -> Option<Duration> {
    end?.checked_sub(start?)
}

#[cfg(unix)]
fn timespec_duration(value: libc::timespec) -> Option<Duration> {
    let seconds = u64::try_from(value.tv_sec).ok()?;
    let nanos = u32::try_from(value.tv_nsec).ok()?;
    (nanos < 1_000_000_000).then(|| Duration::new(seconds, nanos))
}
