use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::time::Instant;

#[derive(Clone, Debug)]
pub enum NativeDownloadState {
    RemoteOnly,
    Downloading { attempt: u32 },
    RetryWaiting { attempt: u32, retry_at: Instant },
    Available(PathBuf),
    Suppressed,
    Rejected,
}

impl NativeDownloadState {
    pub fn initial(cacheable: bool) -> Self {
        if cacheable {
            Self::Downloading { attempt: 1 }
        } else {
            Self::RemoteOnly
        }
    }

    pub fn begin_retry(&mut self, now: Instant) -> bool {
        let Self::RetryWaiting { attempt, retry_at } = self else {
            return false;
        };
        if now < *retry_at {
            return false;
        }
        *self = Self::Downloading {
            attempt: attempt.saturating_add(1),
        };
        true
    }

    pub fn finish(&mut self, path: Option<PathBuf>, retryable: bool, now: Instant) {
        if let Some(path) = path {
            *self = Self::Available(path);
            return;
        }
        let attempt = self.attempt();
        *self = if retryable {
            Self::RetryWaiting {
                attempt,
                retry_at: now + retry_delay(attempt),
            }
        } else {
            Self::Rejected
        };
    }

    pub fn is_downloading(&self) -> bool {
        matches!(self, Self::Downloading { .. })
    }

    pub fn local_path(&self) -> Option<&Path> {
        match self {
            Self::Available(path) => Some(path),
            _ => None,
        }
    }

    pub fn mark_available(&mut self, path: PathBuf) {
        *self = Self::Available(path);
    }

    pub fn restart(&mut self) {
        *self = Self::Downloading { attempt: 1 };
    }

    pub fn suppress(&mut self) {
        *self = Self::Suppressed;
    }

    pub fn is_rejected(&self) -> bool {
        matches!(self, Self::Rejected)
    }

    pub fn attempt(&self) -> u32 {
        match self {
            Self::Downloading { attempt } => *attempt,
            _ => 1,
        }
    }
}

fn retry_delay(attempt: u32) -> Duration {
    let exponent = attempt.saturating_sub(1).min(6);
    Duration::from_secs((1_u64 << exponent).min(60))
}
