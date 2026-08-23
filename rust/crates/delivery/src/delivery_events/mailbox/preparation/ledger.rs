use crate::delivery_events::{
    PlayerPreparationFollowup, PlayerPreparationIngress, PlayerPreparationReport,
};
use ghostr_engine::PostId;
use std::collections::{HashMap, VecDeque};

const ACTIVE_ATTEMPT_CAPACITY: usize = 16;
const ATTEMPT_FENCE_CAPACITY: usize = 16;

#[derive(Clone, Debug, Default)]
pub(super) struct PreparationLedger {
    active: HashMap<PostId, PlayerPreparationReport>,
    fences: VecDeque<AttemptFence>,
    latest_client_epoch: u64,
    latest_capability_generation: Option<u64>,
}

impl PreparationLedger {
    pub(super) fn admit_initial(
        &mut self,
        report: &PlayerPreparationReport,
    ) -> Result<Option<PlayerPreparationReport>, PlayerPreparationIngress> {
        self.admit_identity(report)?;
        if !self.fence_allows(report) {
            return Err(PlayerPreparationIngress::Rejected);
        }
        if !self.active.contains_key(report.post()) && self.active.len() == ACTIVE_ATTEMPT_CAPACITY
        {
            return Err(PlayerPreparationIngress::Saturated);
        }
        if !self.can_record_fence(report) {
            return Err(PlayerPreparationIngress::Saturated);
        }
        let released = self.release_replaced(report)?;
        self.record_fence(report);
        self.active.insert(report.post().clone(), report.clone());
        Ok(released)
    }

    pub(super) fn admit_followup(
        &mut self,
        followup: PlayerPreparationFollowup,
    ) -> Result<PlayerPreparationReport, PlayerPreparationIngress> {
        let Some(admitted) = self.active.get(followup.post()) else {
            return Err(PlayerPreparationIngress::Rejected);
        };
        let report = followup
            .anchor_to(admitted)
            .ok_or(PlayerPreparationIngress::Rejected)?;
        if !report.advances(admitted) {
            return Err(PlayerPreparationIngress::Stale);
        }
        if report.is_terminal() {
            self.active.remove(report.post());
        } else {
            self.active.insert(report.post().clone(), report.clone());
        }
        Ok(report)
    }

    pub(super) fn clear(&mut self) {
        self.active.clear();
        self.fences.clear();
        self.latest_client_epoch = 0;
        self.latest_capability_generation = None;
    }

    pub(super) fn active_len(&self) -> usize {
        self.active.len()
    }

    pub(super) fn latest_client_epoch(&self) -> u64 {
        self.latest_client_epoch
    }

    fn fence_allows(&self, report: &PlayerPreparationReport) -> bool {
        self.fence(report)
            .is_none_or(|fence| report.attempt_generation() > fence.max_attempt)
    }

    fn release_replaced(
        &self,
        report: &PlayerPreparationReport,
    ) -> Result<Option<PlayerPreparationReport>, PlayerPreparationIngress> {
        let Some(known) = self.active.get(report.post()) else {
            return Ok(None);
        };
        if !report.supersedes(known) {
            return Err(PlayerPreparationIngress::Stale);
        }
        known
            .release_for_replacement(report)
            .map(Some)
            .ok_or(PlayerPreparationIngress::Rejected)
    }

    fn admit_identity(
        &mut self,
        report: &PlayerPreparationReport,
    ) -> Result<(), PlayerPreparationIngress> {
        if report.client_epoch() < self.latest_client_epoch {
            return Err(PlayerPreparationIngress::Stale);
        }
        if report.client_epoch() > self.latest_client_epoch {
            self.active.clear();
            self.fences.clear();
            self.latest_client_epoch = report.client_epoch();
            self.latest_capability_generation = Some(report.player_capability_generation());
        }
        (self.latest_capability_generation == Some(report.player_capability_generation()))
            .then_some(())
            .ok_or(PlayerPreparationIngress::Rejected)
    }

    fn can_record_fence(&self, report: &PlayerPreparationReport) -> bool {
        self.fence(report).is_some()
            || self.fences.len() < ATTEMPT_FENCE_CAPACITY
            || self.fences.iter().any(|fence| !self.fence_is_active(fence))
    }

    fn record_fence(&mut self, report: &PlayerPreparationReport) {
        if let Some(index) = self.fence_index(report) {
            self.fences.remove(index);
        } else if self.fences.len() == ATTEMPT_FENCE_CAPACITY {
            let index = self
                .fences
                .iter()
                .position(|fence| !self.fence_is_active(fence))
                .expect("fence capacity checked");
            self.fences.remove(index);
        }
        self.fences.push_back(AttemptFence::capture(report));
    }

    fn fence(&self, report: &PlayerPreparationReport) -> Option<&AttemptFence> {
        self.fences.iter().find(|fence| fence.matches(report))
    }

    fn fence_index(&self, report: &PlayerPreparationReport) -> Option<usize> {
        self.fences.iter().position(|fence| fence.matches(report))
    }

    fn fence_is_active(&self, fence: &AttemptFence) -> bool {
        self.active.values().any(|report| fence.matches(report))
    }
}

#[derive(Clone, Debug)]
struct AttemptFence {
    post: PostId,
    client_epoch: u64,
    max_attempt: u64,
}

impl AttemptFence {
    fn capture(report: &PlayerPreparationReport) -> Self {
        Self {
            post: report.post().clone(),
            client_epoch: report.client_epoch(),
            max_attempt: report.attempt_generation(),
        }
    }

    fn matches(&self, report: &PlayerPreparationReport) -> bool {
        self.post == *report.post() && self.client_epoch == report.client_epoch()
    }
}
