use crate::segmented::cache::StageLease;
use crate::segmented::fetch::{FetchFailure, FetchedObject, ObjectContinuation, OriginTelemetry};
use crate::segmented::prepare::{prepare_complete, PreparedComplete, PreparedObject};
use tokio::sync::oneshot;

#[cfg(test)]
#[path = "prepared/cancellation_test.rs"]
mod cancellation_test;

pub(super) struct PreparedTransfer {
    pub(super) received_bytes: u64,
    pub(super) telemetry: OriginTelemetry,
    pub(super) continuation: Option<ObjectContinuation>,
    pub(super) stage: PreparedStage,
    pub(super) lease: StageLease,
}

pub(super) enum PreparedStage {
    Partial(PreparedObject),
    Complete(PreparedComplete),
}

enum PreparationFailure {
    Superseded,
    Cancelled,
}

pub(super) async fn prepare_transfer(
    mut lease: StageLease,
    fetched: FetchedObject,
    mut cancelled: oneshot::Receiver<()>,
) -> Result<PreparedTransfer, FetchFailure> {
    let received_bytes = fetched.body.len() as u64;
    let telemetry = fetched.telemetry;
    let continuation = fetched.continuation.clone();
    let offset = fetched.offset;
    let object = PreparedObject::from(fetched);
    let stage = if continuation.is_some() {
        PreparedStage::Partial(object)
    } else {
        prepare_terminal(&mut lease, object, offset, &mut cancelled)
            .await
            .map_err(|failure| failure.into_fetch(telemetry, received_bytes))?
    };
    Ok(PreparedTransfer {
        received_bytes,
        telemetry,
        continuation,
        stage,
        lease,
    })
}

async fn prepare_terminal(
    lease: &mut StageLease,
    object: PreparedObject,
    offset: u64,
    cancelled: &mut oneshot::Receiver<()>,
) -> Result<PreparedStage, PreparationFailure> {
    let seed = match offset {
        0 => None,
        _ => Some(
            lease
                .claim_assembly(&object)
                .ok_or(PreparationFailure::Superseded)?,
        ),
    };
    prepare_complete(seed, object, cancelled)
        .await
        .map(PreparedStage::Complete)
        .map_err(|()| PreparationFailure::Cancelled)
}

impl PreparationFailure {
    fn into_fetch(self, telemetry: OriginTelemetry, bytes: u64) -> FetchFailure {
        match self {
            Self::Superseded => FetchFailure::superseded(telemetry, bytes),
            Self::Cancelled => FetchFailure::cancelled_after_response(telemetry, bytes),
        }
    }
}

impl PreparedTransfer {
    pub(super) fn cancelled_failure(self) -> FetchFailure {
        FetchFailure::cancelled_after_response(self.telemetry, self.received_bytes)
    }
}
