use super::{Active, SegmentedDelivery, SegmentedDone, SegmentedLaunch};
use crate::manager::time::unix_time_ms;
use crate::manager::transfers::InternalEvent;
use crate::segmented::cache::{
    StageAdmission, StageFence, StageLease, StageRequest, StageReservation,
};
use crate::segmented::fetch::{
    fetch_stage_tracked, FetchFailure, FetchProgress, SegmentedTraffic, StagedFetch,
};
use crate::segmented::scheduler::prepared::{prepare_transfer, PreparedTransfer};
use std::sync::Arc;

#[cfg(test)]
#[path = "launch/final_assembly_reservation_test.rs"]
mod final_assembly_reservation_test;

struct FetchTask {
    launch: SegmentedLaunch,
    pending: super::progress::Pending,
    priority: ghostr_engine::adaptive::PreemptionAuthority,
    cancelled: tokio::sync::oneshot::Receiver<()>,
    fence: StageFence,
    lease: StageLease,
}

struct FetchAdmission {
    fence: StageFence,
    lease: StageLease,
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
        let Some(admission) = self.admit(&launch) else {
            return false;
        };
        let pending = self
            .pending
            .remove(&launch.post)
            .expect("validated pending HLS stage");
        let priority = self
            .targets
            .iter()
            .find(|target| target.post == launch.post)
            .map(|target| target.priority)
            .expect("pending HLS stage has a focus target");
        let post = launch.post.clone();
        let active = spawn(launch, pending.clone(), priority, admission);
        self.active.insert(post, active);
        true
    }

    fn admit(&self, launch: &SegmentedLaunch) -> Option<FetchAdmission> {
        let pending = self.pending.get(&launch.post)?;
        let reservation = stage_reservation(pending, launch.maximum_bytes)?;
        let fence = stage_fence(pending, launch.maximum_bytes);
        let request = StageAdmission::new(
            launch.post.clone(),
            fence.clone(),
            self.startup_eta_ms,
            reservation,
        );
        let lease = self.cache.admit_stage(request)?;
        Some(FetchAdmission { fence, lease })
    }
}

fn stage_fence(pending: &super::progress::Pending, block_bytes: u64) -> StageFence {
    let request = StageRequest::new(
        pending.url.clone(),
        pending.cursor().next_offset,
        block_bytes,
    );
    StageFence::new(pending.generation, pending.attempt, request)
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
    admission: FetchAdmission,
) -> Active {
    let action = launch.action;
    let committed_until_ms = launch.committed_until_ms;
    let traffic = SegmentedTraffic::new(action, launch.traffic.clone());
    let progress = Arc::new(FetchProgress::new(Some(traffic)));
    let (cancellation, cancelled) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(supervise(
        FetchTask {
            launch,
            pending: pending.clone(),
            priority,
            cancelled,
            fence: admission.fence.clone(),
            lease: admission.lease,
        },
        Arc::clone(&progress),
    ));
    Active {
        action,
        fence: admission.fence,
        pending,
        committed_until_ms,
        network: progress,
        _task: task,
        cancellation: Some(cancellation),
        cancelling: false,
    }
}

async fn supervise(task: FetchTask, progress: Arc<FetchProgress>) {
    let action = task.launch.action;
    let post = task.launch.post.clone();
    let fence = task.fence.clone();
    let resources = task.launch.resources;
    let events = task.launch.events.clone();
    let worker = tokio::spawn(run_fetch(task, Arc::clone(&progress)));
    let outcome = joined_outcome(worker.await, &progress);
    progress.finish_network();
    let _ = events.send(InternalEvent::Segmented(Box::new(SegmentedDone {
        action,
        post,
        fence,
        outcome,
        observed_at_ms: unix_time_ms(),
        resources,
    })));
}

async fn run_fetch(task: FetchTask, progress: Arc<FetchProgress>) -> FetchOutcome {
    let fetched = fetch_stage_tracked(
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
    .await;
    let object = fetched.result?;
    let cancelled = fetched
        .cancellation
        .expect("supervised HLS fetch retains cancellation");
    prepare_transfer(task.lease, object, cancelled).await
}

type FetchOutcome = Result<PreparedTransfer, FetchFailure>;

fn joined_outcome(
    observed: Result<FetchOutcome, tokio::task::JoinError>,
    progress: &FetchProgress,
) -> FetchOutcome {
    match observed {
        Ok(outcome) => outcome,
        Err(error) => Err(FetchFailure::task_failed(error, progress)),
    }
}
