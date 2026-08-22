use crate::transform::{TransformBackend, TransformControl, TransformInput, TransformProfile};
use ghostr_engine::adaptive::TransformKind;
use std::sync::Arc;
use std::time::Duration;

mod cpu_clock;
use cpu_clock::CpuClock;

pub(super) struct Run {
    pub(super) backend: Arc<dyn TransformBackend>,
    pub(super) bytes: Vec<u8>,
    pub(super) kind: TransformKind,
    pub(super) profile: TransformProfile,
    pub(super) control: TransformControl,
}

pub(super) enum Attempt {
    Finished {
        output: Result<Vec<u8>, &'static str>,
        cpu_ms: u64,
    },
    UnmeasuredFailure(&'static str),
}

struct Work {
    output: anyhow::Result<Vec<u8>>,
    cpu: Option<Duration>,
}

pub(super) async fn execute(run: Run) -> Attempt {
    execute_with_clock(run, CpuClock::system()).await
}

#[cfg(test)]
pub(super) async fn execute_without_clock(run: Run) -> Attempt {
    execute_with_clock(run, CpuClock::unavailable()).await
}

async fn execute_with_clock(run: Run, clock: CpuClock) -> Attempt {
    let profile = run.profile;
    let control = run.control.clone();
    let mut work = tokio::task::spawn_blocking(move || run_work(run, clock));
    let limit = Duration::from_millis(profile.limits().elapsed_ms());
    match tokio::time::timeout(limit, &mut work).await {
        Ok(Ok(result)) => finished(result, profile),
        Ok(Err(_)) => Attempt::UnmeasuredFailure("warp_transform_backend_rejected"),
        Err(_) => {
            control.cancel();
            deadline(work.await)
        }
    }
}

fn run_work(run: Run, clock: CpuClock) -> Work {
    let Some(started) = clock.read() else {
        return Work {
            output: Err(anyhow::anyhow!("thread CPU clock unavailable")),
            cpu: None,
        };
    };
    let output = run
        .backend
        .transform(TransformInput::new(run.kind, &run.bytes), &run.control)
        .map(|output| output.into_bytes());
    let cpu = cpu_clock::elapsed(Some(started), clock.read());
    Work { output, cpu }
}

fn finished(work: Work, profile: TransformProfile) -> Attempt {
    let Some(cpu) = work.cpu else {
        return Attempt::UnmeasuredFailure("warp_transform_cpu_measurement_unavailable");
    };
    let output = work
        .output
        .map_err(|_| "warp_transform_backend_rejected")
        .and_then(|bytes| validate(bytes, cpu, profile));
    Attempt::Finished {
        output,
        cpu_ms: duration_ms(cpu),
    }
}

fn validate(
    output: Vec<u8>,
    cpu: Duration,
    profile: TransformProfile,
) -> Result<Vec<u8>, &'static str> {
    if cpu > Duration::from_millis(profile.limits().cpu_ms()) {
        return Err("warp_transform_cpu_envelope_exceeded");
    }
    if output.len() as u64 > profile.limits().output_bytes() {
        return Err("warp_transform_output_envelope_rejected");
    }
    Ok(output)
}

fn deadline(result: Result<Work, tokio::task::JoinError>) -> Attempt {
    let Ok(work) = result else {
        return Attempt::UnmeasuredFailure("warp_transform_backend_rejected");
    };
    let Some(cpu) = work.cpu else {
        return Attempt::UnmeasuredFailure("warp_transform_cpu_measurement_unavailable");
    };
    Attempt::Finished {
        output: Err("warp_transform_deadline_exceeded"),
        cpu_ms: duration_ms(cpu),
    }
}

fn duration_ms(duration: Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}
