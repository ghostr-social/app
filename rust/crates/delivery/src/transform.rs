//! Bounded, injectable media transforms used by selected WARP actions.

use anyhow::{ensure, Result};
use ghostr_engine::adaptive::TransformKind;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

mod fast_start;
pub use fast_start::FastStartRemuxBackend;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransformLimits {
    input_bytes: u64,
    output_bytes: u64,
    cpu_ms: u64,
    elapsed_ms: u64,
}

impl TransformLimits {
    pub fn try_new(
        input_bytes: u64,
        output_bytes: u64,
        cpu_ms: u64,
        elapsed_ms: u64,
    ) -> Result<Self> {
        ensure!(
            input_bytes > 0 && output_bytes > 0,
            "transform byte limits must be positive"
        );
        ensure!(
            cpu_ms > 0 && elapsed_ms > 0,
            "transform time limits must be positive"
        );
        ensure!(
            input_bytes <= 64 << 20 && output_bytes <= 64 << 20,
            "transform byte limits exceed the production bound"
        );
        ensure!(
            cpu_ms <= 1_000 && cpu_ms <= elapsed_ms && elapsed_ms <= 5_000,
            "transform time limits exceed the production bound"
        );
        Ok(Self {
            input_bytes,
            output_bytes,
            cpu_ms,
            elapsed_ms,
        })
    }

    pub const fn input_bytes(self) -> u64 {
        self.input_bytes
    }
    pub const fn output_bytes(self) -> u64 {
        self.output_bytes
    }
    pub const fn cpu_ms(self) -> u64 {
        self.cpu_ms
    }
    pub const fn elapsed_ms(self) -> u64 {
        self.elapsed_ms
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransformProfile {
    kind: TransformKind,
    limits: TransformLimits,
    trigger: TransformTrigger,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransformTrigger {
    Never,
    InvalidVideoTrack,
}

impl TransformProfile {
    pub const fn new(kind: TransformKind, limits: TransformLimits) -> Self {
        Self {
            kind,
            limits,
            trigger: TransformTrigger::Never,
        }
    }
    pub const fn with_trigger(mut self, trigger: TransformTrigger) -> Self {
        self.trigger = trigger;
        self
    }
    pub const fn kind(self) -> TransformKind {
        self.kind
    }
    pub const fn limits(self) -> TransformLimits {
        self.limits
    }
    pub const fn trigger(self) -> TransformTrigger {
        self.trigger
    }
}

impl TransformTrigger {
    pub(crate) fn allows_failure(self, failure: Option<&str>) -> bool {
        matches!(self, Self::InvalidVideoTrack) && failure == Some("invalidVideoTrack")
    }
}

#[derive(Clone, Copy)]
pub struct TransformInput<'a> {
    kind: TransformKind,
    bytes: &'a [u8],
}

impl<'a> TransformInput<'a> {
    pub const fn new(kind: TransformKind, bytes: &'a [u8]) -> Self {
        Self { kind, bytes }
    }
    pub const fn kind(self) -> TransformKind {
        self.kind
    }
    pub const fn bytes(self) -> &'a [u8] {
        self.bytes
    }
}

pub struct TransformOutput(Vec<u8>);

impl TransformOutput {
    pub fn try_new(bytes: Vec<u8>) -> Result<Self> {
        ensure!(!bytes.is_empty(), "transform output must not be empty");
        Ok(Self(bytes))
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

#[derive(Clone)]
pub struct TransformControl {
    cancelled: Arc<AtomicBool>,
    deadline: Instant,
}

impl TransformControl {
    pub fn new(deadline: Instant) -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            deadline,
        }
    }

    pub fn checkpoint(&self) -> Result<()> {
        ensure!(
            !self.cancelled.load(Ordering::Acquire),
            "transform cancelled"
        );
        ensure!(
            Instant::now() <= self.deadline,
            "transform deadline exceeded"
        );
        Ok(())
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }
}

/// A backend must enforce its advertised byte bounds and call
/// [`TransformControl::checkpoint`] often enough to honor cancellation and
/// elapsed limits. Production only installs cooperative in-process backends.
pub trait TransformBackend: Send + Sync {
    fn profile(&self) -> TransformProfile;
    fn transform(
        &self,
        input: TransformInput<'_>,
        control: &TransformControl,
    ) -> Result<TransformOutput>;
}
