use super::{Active, SegmentedDelivery, SegmentedDone, SegmentedLaunch};
use crate::manager::time::unix_time_ms;
use crate::manager::transfers::InternalEvent;
use crate::segmented::fetch::{fetch_stage, SegmentedTraffic, StagedFetch};

impl SegmentedDelivery {
    pub(crate) fn can_start(&self, launch: &SegmentedLaunch) -> bool {
        let Some(pending) = self.pending.get(&launch.post) else {
            return false;
        };
        !self.active.contains_key(&launch.post)
            && pending.stage == launch.stage
            && pending.url == launch.source
            && launch.maximum_bytes == launch.stage.maximum_bytes()
            && launch.committed_until_ms > unix_time_ms()
    }

    pub(crate) fn start(&mut self, launch: SegmentedLaunch) -> bool {
        if !self.can_start(&launch) {
            return false;
        }
        let pending = self
            .pending
            .remove(&launch.post)
            .expect("validated pending HLS stage");
        if !self.cache.mark_stage_preparing(
            &launch.post,
            pending.generation,
            self.startup_eta_ms,
            launch.maximum_bytes,
        ) {
            self.pending.insert(launch.post, pending);
            return false;
        }
        let priority = self
            .targets
            .iter()
            .find(|target| target.post == launch.post)
            .map(|target| target.priority)
            .expect("pending HLS stage has a focus target");
        let post = launch.post.clone();
        let active = spawn(launch, pending.clone(), priority);
        self.active.insert(post, active);
        true
    }
}

fn spawn(
    launch: SegmentedLaunch,
    pending: super::progress::Pending,
    priority: ghostr_engine::adaptive::PreemptionAuthority,
) -> Active {
    let action = launch.action;
    let post = launch.post.clone();
    let generation = pending.generation;
    let committed_until_ms = launch.committed_until_ms;
    let resources = launch.resources;
    let (cancellation, cancelled) = tokio::sync::oneshot::channel();
    let task_pending = pending.clone();
    let task = tokio::spawn(async move {
        let outcome = fetch_stage(StagedFetch {
            requests: &launch.requests,
            stage: task_pending.stage,
            url: &task_pending.url,
            priority,
            committed_until_ms,
            network_status: &launch.network_status,
            cancellation: Some(cancelled),
            traffic: Some(SegmentedTraffic::new(action, launch.traffic.clone())),
        })
        .await;
        let _ = launch.events.send(InternalEvent::Segmented(SegmentedDone {
            action,
            post,
            generation,
            outcome,
            observed_at_ms: unix_time_ms(),
            resources,
        }));
    });
    Active {
        action,
        pending,
        committed_until_ms,
        _task: task,
        cancellation: Some(cancellation),
        cancelling: false,
    }
}
