//! Bounded background execution of representation-fenced local transforms.

use crate::manager::transfers::InternalEvent;
use crate::transform::{TransformBackend, TransformControl, TransformProfile};
use core::time::Duration;
use ghostr_engine::adaptive::TransformKind;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{ActionId, PostId};
use ghostr_partial_store::partial_range_store::{ContentRevision, PartialRangeStore};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::UnboundedSender;

mod cancellation;
#[cfg(test)]
#[path = "../tests/transform_cancellation_ownership_test.rs"]
mod cancellation_ownership_test;
#[cfg(test)]
#[path = "../tests/transform_cpu_busy_sample_test.rs"]
mod cpu_busy_sample_test;
#[cfg(test)]
#[path = "../tests/transform_deadline_ownership_test.rs"]
mod deadline_ownership_test;
mod execution;
mod lifecycle;
#[cfg(test)]
#[path = "../tests/transform_resource_fixture.rs"]
mod resource_test_fixture;
mod resources;
#[cfg(test)]
#[path = "../tests/transform_singleflight_launch_test.rs"]
mod singleflight_test;
#[cfg(test)]
#[path = "../tests/transform_test_fixture.rs"]
mod test_fixture;

pub(crate) struct TransformRequest {
    pub(super) action: ActionId,
    pub(super) binding: RepresentationBinding,
    pub(super) revision: ContentRevision,
    pub(super) total: u64,
    pub(super) kind: TransformKind,
}

pub(crate) use resources::TransformActualResources;

#[derive(Clone, Copy)]
pub(crate) enum TransformTerminal {
    Succeeded(u64),
    Failed(&'static str),
}

pub(crate) struct TransformDone {
    action: ActionId,
    terminal: TransformTerminal,
    actual_resources: Option<TransformActualResources>,
}

struct ActiveTransform {
    post: PostId,
    binding: RepresentationBinding,
    control: TransformControl,
    cancellation_requested: bool,
}

struct TransformFinish {
    post: PostId,
    cancellation_requested: bool,
}

pub(crate) struct TransformJobs {
    backend: Option<Arc<dyn TransformBackend>>,
    events: UnboundedSender<InternalEvent>,
    resources: crate::manager::resource_control::ResourceControl,
    active: HashMap<ActionId, ActiveTransform>,
}

impl TransformJobs {
    pub(super) fn new(
        backend: Option<Arc<dyn TransformBackend>>,
        events: UnboundedSender<InternalEvent>,
        resources: crate::manager::resource_control::ResourceControl,
    ) -> Self {
        Self {
            backend,
            events,
            resources,
            active: HashMap::new(),
        }
    }

    pub(super) fn profile(&self) -> Option<TransformProfile> {
        self.backend.as_ref().map(|backend| backend.profile())
    }

    pub(super) fn contains(&self, post: &PostId) -> bool {
        self.active.values().any(|job| &job.post == post)
    }

    pub(super) fn launch(
        &mut self,
        store: Arc<PartialRangeStore>,
        request: TransformRequest,
    ) -> bool {
        let Some(backend) = self.backend.clone() else {
            return false;
        };
        if !self.active.is_empty() || backend.profile().kind() != request.kind {
            return false;
        }
        let profile = backend.profile();
        let control = transform_control(profile);
        self.remember(&request, control.clone());
        execution::spawn(
            execution::JobContext {
                events: self.events.clone(),
                backend,
                store,
                profile,
                control,
                resources: self.resources.clone(),
            },
            request,
        );
        true
    }

    fn remember(&mut self, request: &TransformRequest, control: TransformControl) {
        self.active.insert(
            request.action,
            ActiveTransform {
                post: request.binding.post().clone(),
                binding: request.binding.clone(),
                control,
                cancellation_requested: false,
            },
        );
    }

    fn finish(&mut self, done: &TransformDone) -> Option<TransformFinish> {
        let job = self.active.remove(&done.action)?;
        Some(TransformFinish {
            post: job.post,
            cancellation_requested: job.cancellation_requested,
        })
    }
}

fn transform_control(profile: TransformProfile) -> TransformControl {
    let now = Instant::now();
    let duration = Duration::from_millis(profile.limits().elapsed_ms());
    TransformControl::new(now.checked_add(duration).unwrap_or(now))
}
