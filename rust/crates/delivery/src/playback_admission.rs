use ghostr_engine::PostId;
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaybackRejection {
    InactiveDelivery,
    StaleSession,
    StaleSequence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaybackAdmission {
    Accepted,
    Rejected(PlaybackRejection),
    IgnoredInactive,
}

impl PlaybackAdmission {
    pub(crate) fn is_accepted(self) -> bool {
        self == Self::Accepted
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PlaybackAdmissionCounters {
    accepted: u64,
    inactive_delivery: u64,
    stale_session: u64,
    stale_sequence: u64,
}

impl PlaybackAdmissionCounters {
    pub fn accepted(self) -> u64 {
        self.accepted
    }

    pub fn rejected(self, reason: PlaybackRejection) -> u64 {
        match reason {
            PlaybackRejection::InactiveDelivery => self.inactive_delivery,
            PlaybackRejection::StaleSession => self.stale_session,
            PlaybackRejection::StaleSequence => self.stale_sequence,
        }
    }

    pub fn total(self) -> u64 {
        self.accepted
            .saturating_add(self.inactive_delivery)
            .saturating_add(self.stale_session)
            .saturating_add(self.stale_sequence)
    }

    fn record(&mut self, admission: PlaybackAdmission) {
        let counter = match admission {
            PlaybackAdmission::Accepted => &mut self.accepted,
            PlaybackAdmission::IgnoredInactive => return,
            PlaybackAdmission::Rejected(PlaybackRejection::InactiveDelivery) => {
                &mut self.inactive_delivery
            }
            PlaybackAdmission::Rejected(PlaybackRejection::StaleSession) => &mut self.stale_session,
            PlaybackAdmission::Rejected(PlaybackRejection::StaleSequence) => {
                &mut self.stale_sequence
            }
        };
        *counter = counter.saturating_add(1);
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PlaybackAdmissionSnapshot {
    counters: PlaybackAdmissionCounters,
    last_accepted: Option<PostId>,
}

impl PlaybackAdmissionSnapshot {
    pub fn counters(&self) -> PlaybackAdmissionCounters {
        self.counters
    }

    pub fn last_accepted(&self) -> Option<&PostId> {
        self.last_accepted.as_ref()
    }

    fn record(&mut self, admission: PlaybackAdmission, post: &PostId) {
        self.counters.record(admission);
        if admission.is_accepted() {
            self.last_accepted = Some(post.clone());
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PlaybackAdmissionLedger {
    snapshot: Arc<Mutex<PlaybackAdmissionSnapshot>>,
}

impl PlaybackAdmissionLedger {
    pub(crate) fn record(&self, admission: PlaybackAdmission, post: &PostId) {
        self.lock().record(admission, post);
    }

    pub(crate) fn snapshot(&self) -> PlaybackAdmissionSnapshot {
        self.lock().clone()
    }

    fn lock(&self) -> MutexGuard<'_, PlaybackAdmissionSnapshot> {
        self.snapshot
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
