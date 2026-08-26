use super::{TransformActualResources, TransformDone, TransformRequest, TransformTerminal};
use crate::manager::transfers::InternalEvent;
use crate::transform::{TransformBackend, TransformControl, TransformProfile};
use ghostr_partial_store::partial_range_store::{
    PartialRangeStore, RepresentationRead, TransformFence, TransformPublication,
    TransformPublicationOutcome,
};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

mod backend;
#[cfg(test)]
#[path = "../../tests/transform_publication_arbitration_test.rs"]
mod publication_arbitration_test;

pub(super) struct JobContext {
    pub(super) events: UnboundedSender<InternalEvent>,
    pub(super) backend: Arc<dyn TransformBackend>,
    pub(super) store: Arc<PartialRangeStore>,
    pub(super) profile: TransformProfile,
    pub(super) control: TransformControl,
    pub(super) resources: crate::manager::resource_control::ResourceControl,
}

struct TransformCompletion {
    terminal: TransformTerminal,
    actual_resources: Option<TransformActualResources>,
}

pub(super) fn spawn(context: JobContext, request: TransformRequest) {
    tokio::spawn(async move {
        let action = request.action;
        let completion = execute(&context, request).await;
        let done = TransformDone {
            action,
            terminal: completion.terminal,
            actual_resources: completion.actual_resources,
        };
        let _ = context.events.send(InternalEvent::Transform(done));
    });
}

async fn execute(context: &JobContext, request: TransformRequest) -> TransformCompletion {
    let input_limit = context.profile.limits().input_bytes();
    let bytes = match read_input(&context.store, &request, input_limit).await {
        Ok(bytes) => bytes,
        Err(class) => return unmeasured_failure(class),
    };
    let attempt = backend::execute(backend::Run {
        backend: std::sync::Arc::clone(&context.backend),
        bytes,
        kind: request.kind,
        profile: context.profile,
        control: context.control.clone(),
        resources: context.resources.clone(),
    })
    .await;
    complete_attempt(context, request, attempt).await
}

async fn complete_attempt(
    context: &JobContext,
    request: TransformRequest,
    attempt: backend::Attempt,
) -> TransformCompletion {
    match attempt {
        backend::Attempt::UnmeasuredFailure(class) => unmeasured_failure(class),
        backend::Attempt::Finished {
            output: Err(class),
            cpu_ms,
        } => measured_failure(class, cpu_ms),
        backend::Attempt::Finished {
            output: Ok(output),
            cpu_ms,
        } => complete_measured(context, request, output, cpu_ms).await,
    }
}

async fn complete_measured(
    context: &JobContext,
    request: TransformRequest,
    output: Vec<u8>,
    cpu_ms: u64,
) -> TransformCompletion {
    if context.control.checkpoint().is_err() {
        return measured_failure("warp_transform_cancelled", cpu_ms);
    }
    let terminal = publish_output(context, request, output).await;
    measured_completion(terminal, cpu_ms)
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
        Ok(RepresentationRead::Present(_) | RepresentationRead::Missing) => {
            Err("warp_transform_input_incomplete")
        }
        Err(_) => Err("warp_transform_input_read_failed"),
    }
}

async fn publish_output(
    context: &JobContext,
    request: TransformRequest,
    output: Vec<u8>,
) -> TransformTerminal {
    let bytes = output.len() as u64;
    let publication = TransformPublication::try_new(
        TransformFence::new(request.binding, request.revision),
        request.kind,
        output,
        context.profile.limits().output_bytes(),
    );
    let Ok(publication) = publication else {
        return TransformTerminal::Failed("warp_transform_output_envelope_rejected");
    };
    let control = context.control.clone();
    let authorize = move || control.try_begin_commit();
    match context
        .store
        .publish_transform_authorized(publication, authorize)
        .await
    {
        Ok(TransformPublicationOutcome::Published) => TransformTerminal::Succeeded(bytes),
        Ok(TransformPublicationOutcome::Superseded) => {
            TransformTerminal::Failed("warp_transform_input_superseded")
        }
        Ok(TransformPublicationOutcome::Cancelled) => {
            TransformTerminal::Failed("warp_transform_cancelled")
        }
        Err(_) => TransformTerminal::Failed("warp_transform_publication_failed"),
    }
}

fn unmeasured_failure(class: &'static str) -> TransformCompletion {
    TransformCompletion {
        terminal: TransformTerminal::Failed(class),
        actual_resources: None,
    }
}

fn measured_failure(class: &'static str, cpu_ms: u64) -> TransformCompletion {
    TransformCompletion {
        terminal: TransformTerminal::Failed(class),
        actual_resources: Some(TransformActualResources::new(cpu_ms, 0)),
    }
}

fn measured_completion(terminal: TransformTerminal, cpu_ms: u64) -> TransformCompletion {
    let stored = match terminal {
        TransformTerminal::Succeeded(bytes) => bytes,
        TransformTerminal::Failed(_) => 0,
    };
    TransformCompletion {
        terminal,
        actual_resources: Some(TransformActualResources::new(cpu_ms, stored)),
    }
}
