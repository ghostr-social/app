//! Bounded background execution of representation-fenced local transforms.

use crate::manager::transfers::InternalEvent;
use crate::transform::{TransformBackend, TransformControl, TransformProfile};
use ghostr_engine::adaptive::TransformKind;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{ActionId, PostId};
use ghostr_partial_store::partial_range_store::{ContentRevision, PartialRangeStore};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedSender;

mod execution;
mod lifecycle;
#[cfg(test)]
#[path = "../tests/transform_singleflight_launch_test.rs"]
mod singleflight_test;

pub(crate) struct TransformRequest {
    pub(crate) action: ActionId,
    pub(crate) binding: RepresentationBinding,
    pub(crate) revision: ContentRevision,
    pub(crate) total: u64,
    pub(crate) kind: TransformKind,
}

pub(crate) enum TransformTerminal {
    Succeeded(u64),
    Failed(&'static str),
}

pub(crate) struct TransformDone {
    pub(crate) action: ActionId,
    pub(crate) terminal: TransformTerminal,
}

struct ActiveTransform {
    post: PostId,
    binding: RepresentationBinding,
    control: TransformControl,
}

pub(crate) struct TransformJobs {
    backend: Option<Arc<dyn TransformBackend>>,
    events: UnboundedSender<InternalEvent>,
    active: HashMap<ActionId, ActiveTransform>,
}

impl TransformJobs {
    pub(crate) fn new(
        backend: Option<Arc<dyn TransformBackend>>,
        events: UnboundedSender<InternalEvent>,
    ) -> Self {
        Self {
            backend,
            events,
            active: HashMap::new(),
        }
    }

    pub(crate) fn profile(&self) -> Option<TransformProfile> {
        self.backend.as_ref().map(|backend| backend.profile())
    }

    pub(crate) fn contains(&self, post: &PostId) -> bool {
        self.active.values().any(|job| &job.post == post)
    }

    pub(crate) fn launch(
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
            },
        );
    }

    pub(crate) fn finish(&mut self, action: ActionId) -> Option<PostId> {
        self.active.remove(&action).map(|job| job.post)
    }

    pub(crate) fn cancel_obsolete(
        &mut self,
        binding: &RepresentationBinding,
    ) -> Vec<(ActionId, PostId)> {
        let obsolete = self
            .active
            .iter()
            .filter(|(_, job)| job.post == *binding.post() && job.binding != *binding)
            .map(|(action, job)| (*action, job.post.clone()))
            .collect();
        self.cancel(obsolete)
    }

    pub(crate) fn retain(&mut self, posts: &HashSet<PostId>) -> Vec<(ActionId, PostId)> {
        let removed = self
            .active
            .iter()
            .filter(|(_, job)| !posts.contains(&job.post))
            .map(|(action, job)| (*action, job.post.clone()))
            .collect();
        self.cancel(removed)
    }

    pub(crate) fn clear(&mut self) -> Vec<(ActionId, PostId)> {
        let removed = self
            .active
            .iter()
            .map(|(action, job)| (*action, job.post.clone()))
            .collect();
        self.cancel(removed)
    }

    fn cancel(&mut self, jobs: Vec<(ActionId, PostId)>) -> Vec<(ActionId, PostId)> {
        for (action, _) in &jobs {
            if let Some(job) = self.active.remove(action) {
                job.control.cancel();
            }
        }
        jobs
    }
}

fn transform_control(profile: TransformProfile) -> TransformControl {
    let now = Instant::now();
    let duration = Duration::from_millis(profile.limits().elapsed_ms());
    TransformControl::new(now.checked_add(duration).unwrap_or(now))
}
