use super::{TransformDone, TransformRequest, TransformTerminal};
use crate::manager::transfers::InternalEvent;
use crate::transform::{TransformBackend, TransformControl, TransformInput, TransformProfile};
use ghostr_partial_store::partial_range_store::{
    PartialRangeStore, RepresentationRead, TransformFence, TransformPublication,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedSender;

pub(super) struct JobContext {
    pub(super) events: UnboundedSender<InternalEvent>,
    pub(super) backend: Arc<dyn TransformBackend>,
    pub(super) store: Arc<PartialRangeStore>,
    pub(super) profile: TransformProfile,
    pub(super) control: TransformControl,
}

struct BackendRun {
    backend: Arc<dyn TransformBackend>,
    bytes: Vec<u8>,
    kind: ghostr_engine::adaptive::TransformKind,
    profile: TransformProfile,
    control: TransformControl,
}

pub(super) fn spawn(context: JobContext, request: TransformRequest) {
    tokio::spawn(async move {
        let action = request.action;
        let terminal = execute(&context, request).await;
        let _ = context
            .events
            .send(InternalEvent::Transform(TransformDone { action, terminal }));
    });
}

async fn execute(context: &JobContext, request: TransformRequest) -> TransformTerminal {
    let input_limit = context.profile.limits().input_bytes();
    let bytes = match read_input(&context.store, &request, input_limit).await {
        Ok(bytes) => bytes,
        Err(class) => return TransformTerminal::Failed(class),
    };
    let run = BackendRun {
        backend: context.backend.clone(),
        bytes,
        kind: request.kind,
        profile: context.profile,
        control: context.control.clone(),
    };
    let output = match run_backend(run).await {
        Ok(output) => output,
        Err(class) => return TransformTerminal::Failed(class),
    };
    if context.control.checkpoint().is_err() {
        return TransformTerminal::Failed("warp_transform_cancelled");
    }
    publish_output(&context.store, request, context.profile, output).await
}

async fn read_input(
    store: &PartialRangeStore,
    request: &TransformRequest,
    maximum: u64,
) -> Result<Vec<u8>, &'static str> {
    if request.total == 0 || request.total > maximum {
        return Err("warp_transform_input_envelope_rejected");
    }
    match store
        .read_for_representation(&request.binding, 0..request.total)
        .await
    {
        Ok(RepresentationRead::Present(bytes)) if bytes.len() as u64 == request.total => Ok(bytes),
        Ok(RepresentationRead::Superseded) => Err("warp_transform_input_superseded"),
        Ok(RepresentationRead::Present(_)) | Ok(RepresentationRead::Missing) => {
            Err("warp_transform_input_incomplete")
        }
        Err(_) => Err("warp_transform_input_read_failed"),
    }
}

async fn run_backend(run: BackendRun) -> Result<Vec<u8>, &'static str> {
    let profile = run.profile;
    let control = run.control;
    let worker_control = control.clone();
    let backend = run.backend;
    let bytes = run.bytes;
    let kind = run.kind;
    let work = tokio::task::spawn_blocking(move || {
        let started = Instant::now();
        let output = backend.transform(TransformInput::new(kind, &bytes), &worker_control)?;
        Ok::<_, anyhow::Error>((output.into_bytes(), started.elapsed()))
    });
    let limit = Duration::from_millis(profile.limits().elapsed_ms());
    let (output, cpu) = match tokio::time::timeout(limit, work).await {
        Ok(Ok(Ok(output))) => output,
        Ok(Ok(Err(_))) | Ok(Err(_)) => return Err("warp_transform_backend_rejected"),
        Err(_) => {
            control.cancel();
            return Err("warp_transform_deadline_exceeded");
        }
    };
    validate_backend_output(output, cpu, profile)
}

fn validate_backend_output(
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

async fn publish_output(
    store: &PartialRangeStore,
    request: TransformRequest,
    profile: TransformProfile,
    output: Vec<u8>,
) -> TransformTerminal {
    let bytes = output.len() as u64;
    let publication = TransformPublication::try_new(
        TransformFence::new(request.binding, request.revision),
        request.kind,
        output,
        profile.limits().output_bytes(),
    );
    let Ok(publication) = publication else {
        return TransformTerminal::Failed("warp_transform_output_envelope_rejected");
    };
    match store.publish_transform(publication).await {
        Ok(true) => TransformTerminal::Succeeded(bytes),
        Ok(false) => TransformTerminal::Failed("warp_transform_input_superseded"),
        Err(_) => TransformTerminal::Failed("warp_transform_publication_failed"),
    }
}
