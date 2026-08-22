use super::{Active, SegmentedDelivery, SegmentedDone, SegmentedLaunch};
use crate::manager::time::unix_time_ms;
use crate::manager::transfers::InternalEvent;
use crate::segmented::cache::StageReservation;
use crate::segmented::fetch::{
    fetch_stage_tracked, FetchFailure, FetchProgress, FetchedObject, SegmentedTraffic, StagedFetch,
};
use std::sync::Arc;

struct FetchTask {
    launch: SegmentedLaunch,
    pending: super::progress::Pending,
    priority: ghostr_engine::adaptive::PreemptionAuthority,
    cancelled: tokio::sync::oneshot::Receiver<()>,
}

impl SegmentedDelivery {
    pub(crate) fn can_start(&self, launch: &SegmentedLaunch) -> bool {
        let Some(pending) = self.pending.get(&launch.post) else {
            return false;
        };
        !self.active.contains_key(&launch.post)
            && self.cache.snapshot(launch.post.as_str()).phase
                != crate::segmented::SegmentedPhase::Failed
            && pending.stage == launch.stage
            && pending.url == launch.source
            && pending.cursor() == launch.cursor
            && pending
                .cursor()
                .block_bytes(launch.stage, launch.maximum_bytes)
                == Some(launch.maximum_bytes)
            && launch.committed_until_ms > unix_time_ms()
    }

    pub(crate) fn start(&mut self, launch: SegmentedLaunch) -> bool {
        if !self.can_start(&launch) {
            return false;
        }
        let reservation = self
            .pending
            .get(&launch.post)
            .and_then(|pending| stage_reservation(pending, launch.maximum_bytes));
        let Some(reservation) = reservation else {
            return false;
        };
        let pending = self
            .pending
            .remove(&launch.post)
            .expect("validated pending HLS stage");
        if !self.cache.mark_stage_preparing(
            &launch.post,
            pending.generation,
            self.startup_eta_ms,
            reservation,
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

fn stage_reservation(
    pending: &super::progress::Pending,
    block_bytes: u64,
) -> Option<StageReservation> {
    let peak = pending.cursor().peak_storage_bytes(block_bytes)?;
    let assembly = peak.checked_sub(block_bytes)?;
    if assembly == 0 {
        Some(StageReservation::block(block_bytes))
    } else {
        StageReservation::final_block(block_bytes, assembly)
    }
}

fn spawn(
    launch: SegmentedLaunch,
    pending: super::progress::Pending,
    priority: ghostr_engine::adaptive::PreemptionAuthority,
) -> Active {
    let action = launch.action;
    let committed_until_ms = launch.committed_until_ms;
    let (cancellation, cancelled) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(supervise(FetchTask {
        launch,
        pending: pending.clone(),
        priority,
        cancelled,
    }));
    Active {
        action,
        pending,
        committed_until_ms,
        _task: task,
        cancellation: Some(cancellation),
        cancelling: false,
    }
}

async fn supervise(task: FetchTask) {
    let action = task.launch.action;
    let post = task.launch.post.clone();
    let generation = task.pending.generation;
    let resources = task.launch.resources;
    let events = task.launch.events.clone();
    let traffic = SegmentedTraffic::new(action, task.launch.traffic.clone());
    let progress = Arc::new(FetchProgress::new(Some(traffic)));
    let worker = tokio::spawn(run_fetch(task, Arc::clone(&progress)));
    let outcome = joined_outcome(worker.await, &progress);
    progress.close_traffic();
    let _ = events.send(InternalEvent::Segmented(Box::new(SegmentedDone {
        action,
        post,
        generation,
        outcome,
        observed_at_ms: unix_time_ms(),
        resources,
    })));
}

async fn run_fetch(task: FetchTask, progress: Arc<FetchProgress>) -> FetchOutcome {
    fetch_stage_tracked(
        StagedFetch {
            requests: &task.launch.requests,
            stage: task.pending.stage,
            url: &task.pending.url,
            maximum_bytes: task.launch.maximum_bytes,
            continuation: task.pending.continuation.as_ref(),
            priority: task.priority,
            committed_until_ms: task.launch.committed_until_ms,
            network_status: &task.launch.network_status,
            cancellation: Some(task.cancelled),
            #[cfg(test)]
            traffic: None,
        },
        &progress,
    )
    .await
}

type FetchOutcome = Result<FetchedObject, FetchFailure>;

fn joined_outcome(
    observed: Result<FetchOutcome, tokio::task::JoinError>,
    progress: &FetchProgress,
) -> FetchOutcome {
    match observed {
        Ok(outcome) => outcome,
        Err(error) => Err(FetchFailure::task_failed(error, progress)),
    }
}
